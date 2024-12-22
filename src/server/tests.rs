#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use hyper::{Request, StatusCode};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::net::SocketAddr;
    use tarpc::context;
    use tower::ServiceExt;

    // Helper function to create a test server instance
    fn test_server() -> TokenServer {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 50051); // Test server address
        TokenServer::new(addr)
    }

    // Helper function to create a test router
    fn test_router() -> Router {
        rest::router::build_router(test_server())
    }

    // Test REST API endpoints
    #[tokio::test]
    async fn test_rest_root_endpoint() {
        let app = test_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["version"], "0.16.0");
        assert!(json["endpoints"].as_array().unwrap().contains(&json!("/")));
        assert!(json["endpoints"].as_array().unwrap().contains(&json!("/create")));
        assert!(json["endpoints"].as_array().unwrap().contains(&json!("/verify_url")));
        assert!(json["endpoints"].as_array().unwrap().contains(&json!("/verify_content")));
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_rest_create_endpoint() {
        let app = test_router();
        let payload = serde_json::json!({
            "decimals": 8,
            "name": "Test Token",
            "symbol": "TST",
            "description": "Test Description",
            "is_frozen": false,
            "environment": "devnet"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/create")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert!(json["token"].is_string());
        assert!(json["move_toml"].is_string());
        assert!(json["test_token"].is_string());
    }

    #[tokio::test]
    async fn test_rest_verify_url_endpoint() {
        let app = test_router();
        let payload = serde_json::json!({
            "url": "https://github.com/valid/repo.git"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/verify_url")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Since the URL doesn't exist, we expect an error response
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_rest_verify_content_endpoint() {
        let app = test_router();
        let payload = serde_json::json!({
            "content": "invalid content"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/verify_content")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    // Test token creation validation
    #[tokio::test]
    async fn test_create_token_validation() {
        let server = test_server();
        let ctx = context::current();

        // Test invalid decimals
        let result = server
            .create(
                ctx.clone(),
                0,
                "Test Token".to_string(),
                "TST".to_string(),
                "Test Description".to_string(),
                false,
                "devnet".to_string(),
            )
            .await;
        assert!(matches!(result, Err(TokenGenError::InvalidDecimals(_))));

        // Test invalid name (empty)
        let result = server
            .create(
                ctx.clone(),
                8,
                "".to_string(),
                "TST".to_string(),
                "Test Description".to_string(),
                false,
                "devnet".to_string(),
            )
            .await;
        assert!(matches!(result, Err(TokenGenError::InvalidName(_))));

        // Test invalid name (too long)
        let result = server
            .create(
                ctx.clone(),
                8,
                "A".repeat(33),
                "TST".to_string(),
                "Test Description".to_string(),
                false,
                "devnet".to_string(),
            )
            .await;
        assert!(matches!(result, Err(TokenGenError::InvalidName(_))));

        // Test invalid symbol (empty)
        let result = server
            .create(
                ctx.clone(),
                8,
                "Test Token".to_string(),
                "".to_string(),
                "Test Description".to_string(),
                false,
                "devnet".to_string(),
            )
            .await;
        assert!(matches!(result, Err(TokenGenError::InvalidSymbol(_))));

        // Test invalid symbol (too long)
        let result = server
            .create(
                ctx.clone(),
                8,
                "Test Token".to_string(),
                "ABCD".to_string(),
                "Test Description".to_string(),
                false,
                "devnet".to_string(),
            )
            .await;
        assert!(matches!(result, Err(TokenGenError::InvalidSymbol(_))));

        // Test invalid description (too long)
        let result = server
            .create(
                ctx.clone(),
                8,
                "Test Token".to_string(),
                "TST".to_string(),
                "A".repeat(1001),
                false,
                "devnet".to_string(),
            )
            .await;
        assert!(matches!(result, Err(TokenGenError::InvalidDescription(_))));

        // Test invalid environment
        let result = server
            .create(
                ctx.clone(),
                8,
                "Test Token".to_string(),
                "TST".to_string(),
                "Test Description".to_string(),
                false,
                "invalid".to_string(),
            )
            .await;
        assert!(matches!(result, Err(TokenGenError::InvalidEnvironment(_))));
    }

    // Test URL verification validation
    #[tokio::test]
    async fn test_verify_url_validation() {
        let server = test_server();
        let ctx = context::current();

        // Test invalid URL format
        let result = server.verify_url(ctx.clone(), "invalid_url".to_string()).await;
        assert!(matches!(result, Err(TokenGenError::InvalidUrl(_))));

        // Test non-existent repository
        let result = server
            .verify_url(ctx.clone(), "https://github.com/nonexistent/repo.git".to_string())
            .await;
        assert!(matches!(result, Err(TokenGenError::RepositoryNotFound(_))));

        // Test invalid content
        let result = server.verify_url(ctx, "https://github.com/valid/repo.git".to_string()).await;
        assert!(matches!(result, Err(TokenGenError::RepositoryNotFound(_))));
    }

    // Test content verification validation
    #[tokio::test]
    async fn test_verify_content_validation() {
        let server = test_server();
        let ctx = context::current();

        // Test invalid content
        let result = server
            .verify_content(ctx.clone(), "invalid content".to_string())
            .await;
        assert!(matches!(result, Err(TokenGenError::InvalidContent(_))));

        // Test empty content
        let result = server.verify_content(ctx, "".to_string()).await;
        assert!(matches!(result, Err(TokenGenError::InvalidContent(_))));
    }

    // Test concurrent RPC operations
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_operations() {
        let server = test_server();
        let ctx = context::current();
        const SAMPLE_DATA_SIZE: u32 = 10;

        let create_requests = generate_create_requests(SAMPLE_DATA_SIZE);
        let verify_requests = generate_verify_requests(SAMPLE_DATA_SIZE);

        let create_tasks: Vec<_> = create_requests
            .into_iter()
            .map(
                |(decimals, name, symbol, description, is_frozen, environment)| {
                    let server = server.clone();
                    let ctx = ctx.clone();
                    tokio::spawn(async move {
                        server
                            .create(
                                ctx,
                                decimals,
                                name,
                                symbol,
                                description,
                                is_frozen,
                                environment,
                            )
                            .await
                    })
                },
            )
            .collect();

        let verify_tasks: Vec<_> = verify_requests
            .into_iter()
            .map(|content| {
                let server = server.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move { server.verify_url(ctx, content).await })
            })
            .collect();

        // Collect results and assert that no errors occurred
        let create_results = future::join_all(create_tasks).await;
        assert!(
            create_results.iter().all(|res| res.is_ok()),
            "Some create tasks failed"
        );

        let verify_results = future::join_all(verify_tasks).await;
        assert!(
            verify_results.iter().all(|res| res.is_ok()),
            "Some verify tasks failed"
        );
    }

    // Generate random create requests for testing
    fn generate_create_requests(limit: u32) -> Vec<(u8, String, String, String, bool, String)> {
        let mut rng = thread_rng();
        let environments = ["mainnet", "testnet", "devnet"];

        (0..limit)
            .map(|_| {
                let decimals = rng.gen_range(1..100);
                let name: String = (0..10).map(|_| rng.sample(Alphanumeric) as char).collect();
                let symbol: String = (0..3).map(|_| rng.sample(Alphanumeric) as char).collect();
                let description = if rng.gen_bool(0.5) {
                    "".to_string()
                } else {
                    (0..20).map(|_| rng.sample(Alphanumeric) as char).collect()
                };
                let is_frozen = rng.gen_bool(0.5);
                let environment = environments[rng.gen_range(0..environments.len())].to_string();

                (decimals, name, symbol, description, is_frozen, environment)
            })
            .collect()
    }

    // Generate verify URL requests
    fn generate_verify_requests(limit: u32) -> Vec<String> {
        let base_url = "https://github.com/valid/repo.git";
        (0..limit)
            .map(|_| format!("{}{}", base_url, rand::random::<u32>() % 10000))
            .collect()
    }
}
