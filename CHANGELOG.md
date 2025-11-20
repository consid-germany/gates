## [1.3.1](https://github.com/consid-germany/gates/compare/v1.3.0...v1.3.1) (2025-11-20)


### Bug Fixes

* **api:** use display_order attribute in create_gate request body ([#554](https://github.com/consid-germany/gates/issues/554)) ([23b06a0](https://github.com/consid-germany/gates/commit/23b06a0f07fa2f8a2446e57c45621bb75cd7b439))

# [1.3.0](https://github.com/consid-germany/gates/compare/v1.2.0...v1.3.0) (2025-03-10)


### Bug Fixes

* **action:** invalid version reference of GitHub Action example in README ([#392](https://github.com/consid-germany/gates/issues/392)) ([d6c1d81](https://github.com/consid-germany/gates/commit/d6c1d81b28e8fcfe852cdf1cd74faad913d664eb))
* **api:** add openssl to api dependencies to resolve cargo lambda build problem ([#430](https://github.com/consid-germany/gates/issues/430)) ([64b0c85](https://github.com/consid-germany/gates/commit/64b0c850f70a379c074ae733a6fb56252bc4c2f6))
* **api:** is_outside_of_business_times ([54dfb7e](https://github.com/consid-germany/gates/commit/54dfb7e5e3f59c8f9823270c91d62ca8bb98f1df))
* **api:** use RFC3339/ISO 8601 compliant date time representations ([#372](https://github.com/consid-germany/gates/issues/372)) ([9f4e112](https://github.com/consid-germany/gates/commit/9f4e1122e6a41fe616613d2ab4dc162b37899de1))
* bump jose from 5.10.0 to 6.0.8 in /cdk ([#418](https://github.com/consid-germany/gates/issues/418)) ([3c44dc2](https://github.com/consid-germany/gates/commit/3c44dc2e8f13d1b1a31cdba9e77c96d0519398bc))
* **cdk:** bump the non-major group across 1 directory with 9 updates ([#54](https://github.com/consid-germany/gates/issues/54)) ([de46288](https://github.com/consid-germany/gates/commit/de462889e7f8da9e22b9ccda87053751f24c250c))
* **cdk:** NPM package does not have a README file ([#391](https://github.com/consid-germany/gates/issues/391)) ([f258e20](https://github.com/consid-germany/gates/commit/f258e200c032551a0d6e1559ce6a0ff1988b9830))
* **cdk:** update 'should match snapshot' cdk unit test affected by dependency updates ([064441f](https://github.com/consid-germany/gates/commit/064441fbebc1b1fe53bee8ff3f69e2aba2463f56))
* **cdk:** use correct path to api and ui build artifacts in copy-builds script ([#390](https://github.com/consid-germany/gates/issues/390)) ([8268714](https://github.com/consid-germany/gates/commit/826871449aac5af9afbd19ada09defe48d98bb71))
* **cdk:** use correct path when copying api and ui builds to build directory ([#432](https://github.com/consid-germany/gates/issues/432)) ([e8b4d56](https://github.com/consid-germany/gates/commit/e8b4d5680cfa01850c53857ce1e63684ee4b8365))
* do not change directory in generate_openapi_models.sh ([c871f61](https://github.com/consid-germany/gates/commit/c871f61b29a18fb72376df9fe106688323d9fa73))
* generate open-api models in build.sh ([#396](https://github.com/consid-germany/gates/issues/396)) ([115621f](https://github.com/consid-germany/gates/commit/115621f73a16a4cf4ad2b3969973ae2b01ade465))
* ignore clippy field is never read false positive ([#157](https://github.com/consid-germany/gates/issues/157)) ([b0510e0](https://github.com/consid-germany/gates/commit/b0510e00d7dc34693b8494af670f6f3a483b07e0))
* **ui:** add flowbite-svelte plugin and sources to app.css ([#434](https://github.com/consid-germany/gates/issues/434)) ([87c1d9c](https://github.com/consid-germany/gates/commit/87c1d9cc3ac88fe99bb7e8c1535dbc8513480c7f))
* **ui:** fixed deserialization error for initial comment message ([#31](https://github.com/consid-germany/gates/issues/31)) ([16bd78c](https://github.com/consid-germany/gates/commit/16bd78cca1d20de56897266a9b1f1a92ff670994))


### Features

* **api:** add random quotes for sanitised comments in demo mode [#202](https://github.com/consid-germany/gates/issues/202) ([f8df2a1](https://github.com/consid-germany/gates/commit/f8df2a135799e758170160dea4528d62e240de2d))
* **api:** use generated open api models ([#8](https://github.com/consid-germany/gates/issues/8)) ([9108b0d](https://github.com/consid-germany/gates/commit/9108b0d14d0b8156deaf9579b6f0a0e71cb64808))

# 1.0.0 (2024-04-11)


### Features

* initial commit ([0624908](https://github.com/consid-germany/gates/commit/0624908ae6969f92fc1684ce98c5ef0e75bcd81d))
