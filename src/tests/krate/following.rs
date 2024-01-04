use crate::builders::CrateBuilder;
use crate::util::{RequestHelper, TestApp};
use googletest::prelude::*;
use http::StatusCode;
use insta::assert_display_snapshot;

fn assert_is_following(crate_name: &str, expected: bool, user: &impl RequestHelper) {
    let response = user.get::<()>(&format!("/api/v1/crates/{crate_name}/following"));
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.json(), json!({ "following": expected }));
}

fn follow(crate_name: &str, user: &impl RequestHelper) {
    let response = user.put::<()>(&format!("/api/v1/crates/{crate_name}/follow"), b"" as &[u8]);
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.json(), json!({ "ok": true }));
}

fn unfollow(crate_name: &str, user: &impl RequestHelper) {
    let response = user.delete::<()>(&format!("/api/v1/crates/{crate_name}/follow"));
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.json(), json!({ "ok": true }));
}

#[test]
fn test_unauthenticated_requests() {
    const CRATE_NAME: &str = "foo";

    let (app, anon, user) = TestApp::init().with_user();

    app.db(|conn| {
        CrateBuilder::new(CRATE_NAME, user.as_model().id).expect_build(conn);
    });

    let response = anon.get::<()>(&format!("/api/v1/crates/{CRATE_NAME}/following"));
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_display_snapshot!(response.text(), @r###"{"errors":[{"detail":"must be logged in to perform that action"}]}"###);

    let response = anon.put::<()>(&format!("/api/v1/crates/{CRATE_NAME}/follow"), b"" as &[u8]);
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_display_snapshot!(response.text(), @r###"{"errors":[{"detail":"must be logged in to perform that action"}]}"###);

    let response = anon.delete::<()>(&format!("/api/v1/crates/{CRATE_NAME}/follow"));
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_display_snapshot!(response.text(), @r###"{"errors":[{"detail":"must be logged in to perform that action"}]}"###);
}

#[test]
fn test_following() {
    const CRATE_NAME: &str = "foo_following";

    let (app, _, user) = TestApp::init().with_user();

    app.db(|conn| {
        CrateBuilder::new(CRATE_NAME, user.as_model().id).expect_build(conn);
    });

    assert_is_following(CRATE_NAME, false, &user);
    follow(CRATE_NAME, &user);
    follow(CRATE_NAME, &user);
    assert_is_following(CRATE_NAME, true, &user);
    assert_that!(user.search("following=1").crates, len(eq(1)));

    unfollow(CRATE_NAME, &user);
    unfollow(CRATE_NAME, &user);
    assert_is_following(CRATE_NAME, false, &user);
    assert_that!(user.search("following=1").crates, empty());
}

#[test]
fn test_api_token_auth() {
    const CRATE_TO_FOLLOW: &str = "some_crate_to_follow";
    const CRATE_NOT_TO_FOLLOW: &str = "another_crate";

    let (app, _, user, token) = TestApp::init().with_token();
    let api_token = token.as_model();

    app.db(|conn| {
        CrateBuilder::new(CRATE_TO_FOLLOW, api_token.user_id).expect_build(conn);
        CrateBuilder::new(CRATE_NOT_TO_FOLLOW, api_token.user_id).expect_build(conn);
    });

    follow(CRATE_TO_FOLLOW, &token);

    // Token auth on GET for get following status is disallowed
    assert_is_following(CRATE_TO_FOLLOW, true, &user);
    assert_is_following(CRATE_NOT_TO_FOLLOW, false, &user);

    let json = token.search("following=1");
    assert_that!(json.crates, len(eq(1)));
}
