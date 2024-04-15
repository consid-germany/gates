use std::sync::Arc;

use axum::routing::{delete, get, post, put};
use axum::Router;
use lambda_http::run;
use lambda_runtime::Error;
use tower_http::trace;
use tower_http::trace::TraceLayer;

use crate::types::app_state::AppState;
use crate::use_cases::{
    add_comment, api_info, create_gate, delete_comment, delete_gate, get_config, get_gate,
    get_gate_state, list_gates, update_display_order, update_gate_state,
};

mod clock;
mod date_time_switch;
mod id_provider;
mod storage;
mod types;
mod use_cases;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .compact()
        .init();

    let result = run(create_router(AppState::new(
        storage::default().await,
        Arc::new(clock::default()),
        Arc::new(id_provider::default()),
        Arc::new(date_time_switch::default()),
    )))
    .await;

    return result;
}

fn create_router(app_state: AppState) -> Router {
    let gates_router = Router::new()
        .route(
            "/",
            get(list_gates::route::handler).post(create_gate::route::handler),
        )
        .route(
            "/:group/:service/:environment",
            get(get_gate::route::handler).delete(delete_gate::route::handler),
        )
        .route(
            "/:group/:service/:environment/state",
            put(update_gate_state::route::handler).get(get_gate_state::route::handler),
        )
        .route(
            "/:group/:service/:environment/display-order",
            put(update_display_order::route::handler),
        )
        .route(
            "/:group/:service/:environment/comments",
            post(add_comment::route::handler),
        )
        .route(
            "/:group/:service/:environment/comments/:comment_id",
            delete(delete_comment::route::handler),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        );
    Router::new().nest(
        "/api/",
        Router::new()
            .route("/", get(api_info::route::handler))
            .route("/config", get(get_config::route::handler))
            .nest("/gates", gates_router)
            .with_state(app_state),
    )
}

#[cfg(test)]
mod integration_tests_lambda {
    use std::sync::Arc;

    use axum::http::StatusCode;
    use chrono::DateTime;
    use http_body_util::BodyExt;
    use lambda_http::Service;
    use openapi::models;
    use testcontainers::clients;
    use testcontainers_modules::dynamodb_local::DynamoDb;

    use crate::clock::MockClock;
    use crate::types::app_state::AppState;
    use crate::{create_router, date_time_switch, id_provider, storage};

    #[tokio::test]
    async fn should_handle_api_gateway_proxy_request() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = storage::test(port).await;

        let now = DateTime::parse_from_rfc3339("2023-06-05T13:00:00+00:00") // monday afternoon
            .expect("failed to parse date");
        let mut mock_clock = MockClock::new();
        mock_clock.expect_now().return_const(now);

        let mut router = create_router(AppState::new(
            Arc::new(dynamodb_storage),
            Arc::new(mock_clock),
            Arc::new(id_provider::default()),
            Arc::new(date_time_switch::default()),
        ));

        // when (create gates)
        let request = include_str!("../tests/data/create_gates_api_gateway_proxy_request.json");
        let request = lambda_http::request::from_str(request)
            .expect("failed to load create_gates_api_gateway_proxy_request");
        let response = router.call(request).await.expect("failed to handle event");

        // then
        assert_eq!(response.status(), StatusCode::OK);

        // when (list gates)
        let request = include_str!("../tests/data/list_gates_api_gateway_proxy_request.json");
        let request = lambda_http::request::from_str(request)
            .expect("failed to load list_gates_api_gateway_proxy_request");
        let response = router.call(request).await.expect("failed to handle event");

        let body = response
            .collect()
            .await
            .expect("failed to collect body")
            .to_bytes();
        let groups = serde_json::from_slice::<Vec<models::Group>>(&body)
            .expect("failed to parse body as json to groups");

        assert_eq!(
            groups,
            vec![models::Group {
                name: "some-group".to_owned(),
                services: vec![models::Service {
                    name: "some-service".to_string(),
                    environments: vec![models::Environment {
                        name: "live".to_owned(),
                        gate: Box::new(models::Gate {
                            group: "some-group".to_owned(),
                            service: "some-service".to_owned(),
                            environment: "live".to_owned(),
                            state: models::GateState::Closed,
                            comments: vec![],
                            last_updated: now.to_utc().to_string(),
                            display_order: None,
                        }),
                    }],
                }],
            }]
        );
    }
}

#[cfg(test)]
mod acceptance_tests {
    use openapi::models::Config;
    use std::sync::Arc;

    use axum::http::StatusCode;
    use axum_test::TestServer;
    use chrono::{DateTime, FixedOffset, Utc};
    use openapi::models;
    use testcontainers::clients;
    use testcontainers_modules::dynamodb_local::DynamoDb;

    use crate::clock::MockClock;
    use crate::id_provider::MockIdProvider;
    use crate::types::app_state::AppState;
    use crate::types::GateState;
    use crate::{create_router, date_time_switch, id_provider, storage, types, use_cases};

    fn inside_active_hours() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2023-06-05T13:00:00+00:00") // monday afternoon
            .expect("failed to parse date")
            .into()
    }

    fn outside_active_hours() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2023-06-04T13:00:00+00:00") // sunday afternoon
            .expect("failed to parse date")
            .into()
    }

    #[tokio::test]
    async fn should_create_and_list_gates() {
        // given
        let now = inside_active_hours();
        let mut mock_clock = MockClock::new();
        mock_clock.expect_now().return_const(now);

        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = storage::test(port).await;
        let router = create_router(AppState::new(
            Arc::new(dynamodb_storage),
            Arc::new(mock_clock),
            Arc::new(id_provider::default()),
            Arc::new(date_time_switch::default()),
        ));

        let server = TestServer::new(router).expect("failed to create test server");

        // when
        let response = server
            .post("/api/gates")
            .json(&crate::use_cases::create_gate::route::Payload {
                group: "somegroup".to_owned(),
                service: "someservice".to_owned(),
                environment: "develop".to_owned(),
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server
            .post("/api/gates")
            .json(&crate::use_cases::create_gate::route::Payload {
                group: "somegroup".to_owned(),
                service: "someservice".to_owned(),
                environment: "live".to_owned(),
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server.get("/api/gates").await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Vec<models::Group>>(),
            vec![models::Group {
                name: "somegroup".to_string(),
                services: vec![models::Service {
                    name: "someservice".to_string(),
                    environments: vec![
                        models::Environment {
                            name: "develop".to_string(),
                            gate: Box::new(models::Gate {
                                group: "somegroup".to_string(),
                                service: "someservice".to_string(),
                                environment: "develop".to_string(),
                                state: models::GateState::Closed,
                                comments: vec![],
                                last_updated: now.to_string(),
                                display_order: Option::default(),
                            }),
                        },
                        models::Environment {
                            name: "live".to_string(),
                            gate: Box::new(models::Gate {
                                group: "somegroup".to_string(),
                                service: "someservice".to_string(),
                                environment: "live".to_string(),
                                state: models::GateState::Closed,
                                comments: vec![],
                                last_updated: now.to_string(),
                                display_order: Option::default(),
                            }),
                        },
                    ],
                }],
            }]
        );
    }

    #[tokio::test]
    async fn should_open_and_close_gates() {
        // given
        let now = inside_active_hours();
        let mut mock_clock = MockClock::new();
        mock_clock.expect_now().return_const(now);

        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = storage::test(port).await;
        let router = create_router(AppState::new(
            Arc::new(dynamodb_storage),
            Arc::new(mock_clock),
            Arc::new(id_provider::default()),
            Arc::new(date_time_switch::default()),
        ));

        let server = TestServer::new(router).expect("failed to create test server");

        let group = "somegroup".to_owned();
        let service = "someservice".to_owned();
        let environment = "develop".to_owned();
        let response = server
            .post("/api/gates")
            .json(&use_cases::create_gate::route::Payload {
                group,
                service,
                environment,
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let group = "somegroup".to_owned();
        let service = "someservice".to_owned();
        let environment = "live".to_owned();
        let response = server
            .post("/api/gates")
            .json(&use_cases::create_gate::route::Payload {
                group,
                service,
                environment,
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        // when
        let response = server
            .put("/api/gates/somegroup/someservice/develop/state")
            .json(&crate::use_cases::update_gate_state::route::Payload {
                state: GateState::Open,
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<models::Gate>(),
            models::Gate {
                group: "somegroup".to_string(),
                service: "someservice".to_string(),
                environment: "develop".to_string(),
                state: models::GateState::Open,
                comments: vec![],
                last_updated: now.to_string(),
                display_order: Option::default(),
            }
        );

        let response = server
            .put("/api/gates/somegroup/someservice/develop/state")
            .json(&crate::use_cases::update_gate_state::route::Payload {
                state: GateState::Closed,
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<models::Gate>(),
            models::Gate {
                group: "somegroup".to_string(),
                service: "someservice".to_string(),
                environment: "develop".to_string(),
                state: models::GateState::Closed,
                comments: vec![],
                last_updated: now.to_string(),
                display_order: Option::default(),
            }
        );

        let response = server.get("/api/gates/somegroup/someservice/develop").await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<models::Gate>(),
            models::Gate {
                group: "somegroup".to_string(),
                service: "someservice".to_string(),
                environment: "develop".to_string(),
                state: models::GateState::Closed,
                comments: vec![],
                last_updated: now.to_string(),
                display_order: Option::default(),
            }
        );
    }

    #[tokio::test]
    async fn should_delete_gates() {
        // given
        let now = inside_active_hours();
        let mut mock_clock = MockClock::new();
        mock_clock.expect_now().return_const(now);

        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = storage::test(port).await;
        let router = create_router(AppState::new(
            Arc::new(dynamodb_storage),
            Arc::new(mock_clock),
            Arc::new(id_provider::default()),
            Arc::new(date_time_switch::default()),
        ));

        let server = TestServer::new(router).expect("failed to create test server");

        let group = "somegroup".to_owned();
        let service = "someservice".to_owned();
        let environment = "develop".to_owned();
        let response = server
            .post("/api/gates")
            .json(&use_cases::create_gate::route::Payload {
                group,
                service,
                environment,
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let group = "somegroup".to_owned();
        let service = "someservice".to_owned();
        let environment = "live".to_owned();
        let response = server
            .post("/api/gates")
            .json(&use_cases::create_gate::route::Payload {
                group,
                service,
                environment,
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        // when
        let response = server.delete("/api/gates/somegroup/someservice/live").await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server.get("/api/gates").await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Vec<models::Group>>(),
            vec![models::Group {
                name: "somegroup".to_string(),
                services: vec![models::Service {
                    name: "someservice".to_string(),
                    environments: vec![models::Environment {
                        name: "develop".to_string(),
                        gate: Box::new(models::Gate {
                            group: "somegroup".to_string(),
                            service: "someservice".to_string(),
                            environment: "develop".to_string(),
                            state: models::GateState::Closed,
                            comments: vec![],
                            last_updated: now.to_string(),
                            display_order: Option::default(),
                        }),
                    },],
                }],
            }]
        );
    }

    #[tokio::test]
    async fn should_add_and_remove_comments() {
        // given
        let now = inside_active_hours();
        let mut mock_clock = MockClock::new();
        mock_clock.expect_now().return_const(now);

        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = storage::test(port).await;
        let mut mock_id_provider = MockIdProvider::new();
        mock_id_provider.expect_get().return_const("some_id");
        let router = create_router(AppState::new(
            Arc::new(dynamodb_storage),
            Arc::new(mock_clock),
            Arc::new(mock_id_provider),
            Arc::new(date_time_switch::default()),
        ));

        let server = TestServer::new(router).expect("failed to create test server");

        let response = server.get("/api/gates").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(response.json::<Vec<models::Group>>(), vec![]);

        // when
        let response = server
            .post("/api/gates")
            .json(&crate::use_cases::create_gate::route::Payload {
                group: "somegroup".to_owned(),
                service: "someservice".to_owned(),
                environment: "develop".to_owned(),
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server
            .post("/api/gates/somegroup/someservice/develop/comments")
            .json(&crate::use_cases::add_comment::route::Payload {
                message: "Some comment message".to_owned(),
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server.get("/api/gates").await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Vec<models::Group>>(),
            vec![models::Group {
                name: "somegroup".to_owned(),
                services: vec![models::Service {
                    name: "someservice".to_owned(),
                    environments: vec![models::Environment {
                        name: "develop".to_owned(),
                        gate: Box::new(models::Gate {
                            group: "somegroup".to_owned(),
                            service: "someservice".to_owned(),
                            environment: "develop".to_owned(),
                            state: models::GateState::Closed,
                            comments: vec![models::Comment {
                                id: "some_id".to_owned(),
                                message: "Some comment message".to_owned(),
                                created: now.to_string(),
                            }],
                            last_updated: now.to_string(),
                            display_order: Option::default(),
                        }),
                    },],
                }],
            }]
        );

        // when
        let response = server
            .delete("/api/gates/somegroup/someservice/develop/comments/some_id")
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server.get("/api/gates").await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Vec<models::Group>>(),
            vec![models::Group {
                name: "somegroup".to_string(),
                services: vec![models::Service {
                    name: "someservice".to_string(),
                    environments: vec![models::Environment {
                        name: "develop".to_string(),
                        gate: Box::new(models::Gate {
                            group: "somegroup".to_string(),
                            service: "someservice".to_string(),
                            environment: "develop".to_string(),
                            state: models::GateState::Closed,
                            comments: vec![],
                            last_updated: now.to_string(),
                            display_order: Option::default(),
                        }),
                    },],
                }],
            }]
        );
    }

    #[tokio::test]
    async fn should_get_gate_state() {
        // given
        let now = inside_active_hours();
        let mut mock_clock = MockClock::new();
        mock_clock.expect_now().return_const(now);

        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = storage::test(port).await;
        let router = create_router(AppState::new(
            Arc::new(dynamodb_storage),
            Arc::new(mock_clock),
            Arc::new(id_provider::default()),
            Arc::new(date_time_switch::default()),
        ));

        let server = TestServer::new(router).expect("failed to create test server");

        let group = "somegroup".to_owned();
        let service = "someservice".to_owned();
        let environment = "develop".to_owned();
        let response = server
            .post("/api/gates")
            .json(&use_cases::create_gate::route::Payload {
                group,
                service,
                environment,
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let group = "somegroup".to_owned();
        let service = "someservice".to_owned();
        let environment = "live".to_owned();
        let response = server
            .post("/api/gates")
            .json(&use_cases::create_gate::route::Payload {
                group,
                service,
                environment,
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        // when
        let response = server
            .put("/api/gates/somegroup/someservice/live/state")
            .json(&crate::use_cases::update_gate_state::route::Payload {
                state: GateState::Open,
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server
            .get("/api/gates/somegroup/someservice/live/state")
            .await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<models::GateStateRep>(),
            models::GateStateRep {
                state: models::GateState::Open,
            },
        );
    }

    #[tokio::test]
    async fn should_auto_close_gates() {
        // given
        let now = outside_active_hours();
        let mut mock_clock = MockClock::new();
        mock_clock.expect_now().return_const(now);

        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = storage::test(port).await;
        let router = create_router(AppState::new(
            Arc::new(dynamodb_storage),
            Arc::new(mock_clock),
            Arc::new(id_provider::default()),
            Arc::new(date_time_switch::default()),
        ));

        let server = TestServer::new(router).expect("failed to create test server");

        let group = "somegroup".to_owned();
        let service = "someservice".to_owned();
        let environment = "develop".to_owned();
        let response = server
            .post("/api/gates")
            .json(&use_cases::create_gate::route::Payload {
                group,
                service,
                environment,
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let group = "somegroup".to_owned();
        let service = "someservice".to_owned();
        let environment = "live".to_owned();
        let response = server
            .post("/api/gates")
            .json(&use_cases::create_gate::route::Payload {
                group,
                service,
                environment,
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        // when
        // try to set state of live - CLOSED on sunday
        let response = server
            .put("/api/gates/somegroup/someservice/live/state")
            .json(&crate::use_cases::update_gate_state::route::Payload {
                state: GateState::Open,
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::CONFLICT);
        assert_eq!(
            response.json::<String>(),
            "Already after business hours - rejecting attempt to change state".to_owned()
        );

        let response = server.get("/api/gates/somegroup/someservice/live").await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<models::Gate>(),
            models::Gate {
                group: "somegroup".to_string(),
                service: "someservice".to_string(),
                environment: "live".to_string(),
                state: models::GateState::Closed,
                comments: vec![],
                last_updated: now.to_string(),
                display_order: Option::default(),
            },
        );
    }

    #[tokio::test]
    async fn should_get_config() {
        // given
        let now = inside_active_hours();
        let mut mock_clock = MockClock::new();
        mock_clock.expect_now().return_const(now);

        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = storage::test(port).await;
        let router = create_router(AppState::new(
            Arc::new(dynamodb_storage),
            Arc::new(mock_clock),
            Arc::new(id_provider::default()),
            Arc::new(date_time_switch::default()),
        ));

        let server = TestServer::new(router).expect("failed to create test server");

        // when
        // try to set get the config with the system_time
        let response = server.get("/api/config").await;

        let openapi_active_hours_per_week: models::ActiveHoursPerWeek =
            types::ActiveHoursPerWeek::default().into();
        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Config>(),
            Config::new(now.to_string(), openapi_active_hours_per_week)
        );
    }
    #[tokio::test]
    async fn should_set_display_order() {
        // given
        let now = inside_active_hours();
        let mut mock_clock = MockClock::new();
        mock_clock.expect_now().return_const(now);

        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = storage::test(port).await;
        let router = create_router(AppState::new(
            Arc::new(dynamodb_storage),
            Arc::new(mock_clock),
            Arc::new(id_provider::default()),
            Arc::new(date_time_switch::default()),
        ));

        let server = TestServer::new(router).expect("failed to create test server");

        let group = "somegroup".to_owned();
        let service = "someservice".to_owned();
        let environment = "develop".to_owned();
        let response = server
            .post("/api/gates")
            .json(&use_cases::create_gate::route::Payload {
                group,
                service,
                environment,
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let group = "somegroup".to_owned();
        let service = "someservice".to_owned();
        let environment = "live".to_owned();
        let response = server
            .post("/api/gates")
            .json(&use_cases::create_gate::route::Payload {
                group,
                service,
                environment,
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        // when
        let response = server
            .put("/api/gates/somegroup/someservice/develop/display-order")
            .json(&crate::use_cases::update_display_order::route::Payload { display_order: 1 })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<models::Gate>(),
            models::Gate {
                group: "somegroup".to_string(),
                service: "someservice".to_string(),
                environment: "develop".to_string(),
                state: models::GateState::Closed,
                comments: vec![],
                last_updated: now.to_string(),
                display_order: Some(1f64),
            }
        );

        // then
        let response = server.get("/api/gates").await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Vec<models::Group>>(),
            vec![models::Group {
                name: "somegroup".to_string(),
                services: vec![models::Service {
                    name: "someservice".to_string(),
                    environments: vec![
                        models::Environment {
                            name: "live".to_string(),
                            gate: Box::new(expected_gate_representation(
                                now.into(),
                                "live".to_string()
                            )),
                        },
                        models::Environment {
                            name: "develop".to_string(),
                            gate: Box::new(expected_gate_representation_with_display_order(
                                now.into(),
                                "develop".to_string(),
                                1,
                            )),
                        },
                    ],
                }],
            }]
        );
    }

    fn expected_gate_representation(
        now: DateTime<FixedOffset>,
        environment: String,
    ) -> models::Gate {
        models::Gate {
            group: "somegroup".to_string(),
            service: "someservice".to_string(),
            environment,
            state: models::GateState::Closed,
            comments: vec![],
            last_updated: now.to_utc().to_string(),
            display_order: Option::default(),
        }
    }

    fn expected_gate_representation_with_display_order(
        now: DateTime<FixedOffset>,
        environment: String,
        display_order: u32,
    ) -> models::Gate {
        models::Gate {
            group: "somegroup".to_string(),
            service: "someservice".to_string(),
            environment,
            state: models::GateState::Closed,
            comments: vec![],
            last_updated: now.to_utc().to_string(),
            display_order: Some(display_order as f64),
        }
    }
}
