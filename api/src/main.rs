use axum::routing::{delete, get, post, put};
use axum::Router;
use lambda_http::run;
use lambda_runtime::Error;
use std::sync::Arc;
use tower_http::trace;
use tower_http::trace::TraceLayer;

use crate::types::appstate::AppState;
use crate::use_cases::{
    add_comment, create_gate, delete_comment, delete_gate, get_gate, list_gates,
    update_display_order, update_gate_state,
};

mod clock;
mod date_time_switch;
mod id_provider;
mod routes;
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
        Arc::new(storage::default().await),
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
            put(update_gate_state::route::handler),
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
        .with_state(app_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        );

    Router::new().nest(
        "/:stage/api/",
        Router::new()
            .route("/", get(routes::api_info::handler))
            .nest("/gates", gates_router),
    )
}

#[cfg(test)]
mod integration_tests_lambda {
    use axum::http::StatusCode;
    use std::sync::Arc;

    use chrono::DateTime;
    use http_body_util::BodyExt;
    use lambda_http::Service;
    use testcontainers::clients;
    use testcontainers_modules::dynamodb_local::DynamoDb;

    use crate::clock::MockClock;
    use crate::types::appstate::AppState;
    use crate::types::{representation, GateState};
    use crate::{create_router, date_time_switch, id_provider, storage};

    #[tokio::test]
    async fn should_handle_api_gateway_proxy_request() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = storage::test(port).await;

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
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
        let groups = serde_json::from_slice::<Vec<representation::Group>>(&body)
            .expect("failed to parse body as json to groups");

        assert_eq!(
            groups,
            vec![representation::Group {
                name: "some-group".to_owned(),
                services: vec![representation::Service {
                    name: "some-service".to_string(),
                    environments: vec![representation::Environment {
                        name: "live".to_owned(),
                        gate: representation::Gate {
                            group: "some-group".to_owned(),
                            service: "some-service".to_owned(),
                            environment: "live".to_owned(),
                            state: GateState::default(),
                            comments: vec![],
                            last_updated: now.into(),
                            display_order: None,
                        },
                    }],
                }],
            }]
        );
    }
}

#[cfg(test)]
mod acceptance_tests {
    use axum::http::StatusCode;
    use std::sync::Arc;

    use axum_test::TestServer;
    use chrono::{DateTime, FixedOffset};
    use testcontainers::clients;
    use testcontainers_modules::dynamodb_local::DynamoDb;

    use crate::clock::MockClock;
    use crate::id_provider::MockIdProvider;
    use crate::types::appstate::AppState;
    use crate::types::representation;
    use crate::types::GateState;
    use crate::{create_router, date_time_switch, id_provider, storage};

    #[tokio::test]
    async fn should_create_and_list_gates() {
        // given
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
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
            .post("/test/api/gates")
            .json(&crate::use_cases::create_gate::route::Payload {
                group: "somegroup".to_owned(),
                service: "someservice".to_owned(),
                environment: "develop".to_owned(),
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server
            .post("/test/api/gates")
            .json(&crate::use_cases::create_gate::route::Payload {
                group: "somegroup".to_owned(),
                service: "someservice".to_owned(),
                environment: "live".to_owned(),
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server.get("/test/api/gates").await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Vec<representation::Group>>(),
            vec![representation::Group {
                name: "somegroup".to_string(),
                services: vec![representation::Service {
                    name: "someservice".to_string(),
                    environments: vec![
                        representation::Environment {
                            name: "develop".to_string(),
                            gate: representation::Gate {
                                group: "somegroup".to_string(),
                                service: "someservice".to_string(),
                                environment: "develop".to_string(),
                                state: GateState::default(),
                                comments: vec![],
                                last_updated: now.into(),
                                display_order: Option::default(),
                            },
                        },
                        representation::Environment {
                            name: "live".to_string(),
                            gate: representation::Gate {
                                group: "somegroup".to_string(),
                                service: "someservice".to_string(),
                                environment: "live".to_string(),
                                state: GateState::default(),
                                comments: vec![],
                                last_updated: now.into(),
                                display_order: Option::default(),
                            },
                        },
                    ],
                }],
            }]
        );
    }

    #[tokio::test]
    async fn should_open_and_close_gates() {
        // given
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
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

        initialize_two_gates(&server).await;

        let response = server.get("/test/api/gates").await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Vec<representation::Group>>(),
            vec![representation::Group {
                name: "somegroup".to_string(),
                services: vec![representation::Service {
                    name: "someservice".to_string(),
                    environments: vec![
                        representation::Environment {
                            name: "develop".to_string(),
                            gate: expected_gate_representation(now, "develop".to_string()),
                        },
                        representation::Environment {
                            name: "live".to_string(),
                            gate: expected_gate_representation(now, "live".to_string()),
                        },
                    ],
                }],
            }]
        );

        let response = server
            .put("/test/api/gates/somegroup/someservice/develop/state")
            .json(&crate::use_cases::update_gate_state::route::Payload {
                state: GateState::Open,
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<representation::Gate>(),
            representation::Gate {
                group: "somegroup".to_string(),
                service: "someservice".to_string(),
                environment: "develop".to_string(),
                state: GateState::Open,
                comments: vec![],
                last_updated: now.into(),
                display_order: Option::default(),
            }
        );

        let response = server
            .put("/test/api/gates/somegroup/someservice/develop/state")
            .json(&crate::use_cases::update_gate_state::route::Payload {
                state: GateState::Closed,
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<representation::Gate>(),
            representation::Gate {
                group: "somegroup".to_string(),
                service: "someservice".to_string(),
                environment: "develop".to_string(),
                state: GateState::Closed,
                comments: vec![],
                last_updated: now.into(),
                display_order: Option::default(),
            }
        );

        let response = server
            .get("/test/api/gates/somegroup/someservice/develop")
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<representation::Gate>(),
            representation::Gate {
                group: "somegroup".to_string(),
                service: "someservice".to_string(),
                environment: "develop".to_string(),
                state: GateState::default(),
                comments: vec![],
                last_updated: now.into(),
                display_order: Option::default(),
            }
        );
    }

    #[tokio::test]
    async fn should_delete_gates() {
        // given
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
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

        initialize_two_gates(&server).await;

        // when
        let response = server
            .delete("/test/api/gates/somegroup/someservice/live")
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server.get("/test/api/gates").await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Vec<representation::Group>>(),
            vec![representation::Group {
                name: "somegroup".to_string(),
                services: vec![representation::Service {
                    name: "someservice".to_string(),
                    environments: vec![representation::Environment {
                        name: "develop".to_string(),
                        gate: representation::Gate {
                            group: "somegroup".to_string(),
                            service: "someservice".to_string(),
                            environment: "develop".to_string(),
                            state: GateState::default(),
                            comments: vec![],
                            last_updated: now.into(),
                            display_order: Option::default(),
                        },
                    },],
                }],
            }]
        );
    }

    #[tokio::test]
    async fn should_add_and_remove_comments() {
        // given
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
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

        let response = server.get("/test/api/gates").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(response.json::<Vec<representation::Group>>(), vec![]);

        // when
        let response = server
            .post("/test/api/gates")
            .json(&crate::use_cases::create_gate::route::Payload {
                group: "somegroup".to_owned(),
                service: "someservice".to_owned(),
                environment: "develop".to_owned(),
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server
            .post("/test/api/gates/somegroup/someservice/develop/comments")
            .json(&crate::use_cases::add_comment::route::Payload {
                message: "Some comment message".to_owned(),
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server.get("/test/api/gates").await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Vec<representation::Group>>(),
            vec![representation::Group {
                name: "somegroup".to_owned(),
                services: vec![representation::Service {
                    name: "someservice".to_owned(),
                    environments: vec![representation::Environment {
                        name: "develop".to_owned(),
                        gate: representation::Gate {
                            group: "somegroup".to_owned(),
                            service: "someservice".to_owned(),
                            environment: "develop".to_owned(),
                            state: GateState::default(),
                            comments: vec![representation::Comment {
                                id: "some_id".to_owned(),
                                message: "Some comment message".to_owned(),
                                created: now.into(),
                            }],
                            last_updated: now.into(),
                            display_order: Option::default(),
                        },
                    },],
                }],
            }]
        );

        // when
        let response = server
            .delete("/test/api/gates/somegroup/someservice/develop/comments/some_id")
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server.get("/test/api/gates").await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Vec<representation::Group>>(),
            vec![representation::Group {
                name: "somegroup".to_string(),
                services: vec![representation::Service {
                    name: "someservice".to_string(),
                    environments: vec![representation::Environment {
                        name: "develop".to_string(),
                        gate: representation::Gate {
                            group: "somegroup".to_string(),
                            service: "someservice".to_string(),
                            environment: "develop".to_string(),
                            state: GateState::default(),
                            comments: vec![],
                            last_updated: now.into(),
                            display_order: Option::default(),
                        },
                    },],
                }],
            }]
        );
    }

    #[tokio::test]
    async fn should_auto_close_gates() {
        // given
        let now = DateTime::parse_from_rfc3339("2023-05-28T22:10:57+02:00") //sunday
            .expect("failed to parse date");
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

        initialize_two_gates(&server).await;

        // when
        // try to set state of live - CLOSED on sunday
        let response = server
            .put("/test/api/gates/somegroup/someservice/live/state")
            .json(&crate::use_cases::update_gate_state::route::Payload {
                state: GateState::Open,
            })
            .await;

        assert_eq!(response.status_code(), StatusCode::CONFLICT);
        assert_eq!(
            response.json::<String>(),
            "Already after business hours - rejecting attempt to change state".to_owned()
        );

        let response = server
            .get("/test/api/gates/somegroup/someservice/live")
            .await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<representation::Gate>(),
            representation::Gate {
                group: "somegroup".to_string(),
                service: "someservice".to_string(),
                environment: "live".to_string(),
                state: GateState::Closed,
                comments: vec![],
                last_updated: now.into(),
                display_order: Option::default(),
            },
        );
    }

    #[tokio::test]
    async fn should_set_display_order() {
        // given
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
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

        initialize_two_gates(&server).await;

        // when
        let response = server
            .put("/test/api/gates/somegroup/someservice/develop/display-order")
            .json(&crate::use_cases::update_display_order::route::Payload { display_order: 1 })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<representation::Gate>(),
            representation::Gate {
                group: "somegroup".to_string(),
                service: "someservice".to_string(),
                environment: "develop".to_string(),
                state: GateState::default(),
                comments: vec![],
                last_updated: now.into(),
                display_order: Some(1),
            }
        );

        // then
        let response = server.get("/test/api/gates").await;

        // then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.json::<Vec<representation::Group>>(),
            vec![representation::Group {
                name: "somegroup".to_string(),
                services: vec![representation::Service {
                    name: "someservice".to_string(),
                    environments: vec![
                        representation::Environment {
                            name: "live".to_string(),
                            gate: expected_gate_representation(now, "live".to_string()),
                        },
                        representation::Environment {
                            name: "develop".to_string(),
                            gate: expected_gate_representation_with_display_order(
                                now,
                                "develop".to_string(),
                                1
                            ),
                        },
                    ],
                }],
            }]
        );
    }

    async fn initialize_two_gates(server: &TestServer) {
        // when
        let response = server
            .post("/test/api/gates")
            .json(&crate::use_cases::create_gate::route::Payload {
                group: "somegroup".to_owned(),
                service: "someservice".to_owned(),
                environment: "develop".to_owned(),
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let response = server
            .post("/test/api/gates")
            .json(&crate::use_cases::create_gate::route::Payload {
                group: "somegroup".to_owned(),
                service: "someservice".to_owned(),
                environment: "live".to_owned(),
            })
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);
    }

    fn expected_gate_representation(
        now: DateTime<FixedOffset>,
        environment: String,
    ) -> representation::Gate {
        representation::Gate {
            group: "somegroup".to_string(),
            service: "someservice".to_string(),
            environment,
            state: GateState::default(),
            comments: vec![],
            last_updated: now.into(),
            display_order: Option::default(),
        }
    }

    fn expected_gate_representation_with_display_order(
        now: DateTime<FixedOffset>,
        environment: String,
        display_order: u32,
    ) -> representation::Gate {
        representation::Gate {
            group: "somegroup".to_string(),
            service: "someservice".to_string(),
            environment,
            state: GateState::default(),
            comments: vec![],
            last_updated: now.into(),
            display_order: Some(display_order),
        }
    }
}
