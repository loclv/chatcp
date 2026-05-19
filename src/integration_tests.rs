#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use worker::*;
    use crate::models::*;
    use crate::router;

    // Note: Testing with a real D1 requires miniflare/wrangler.
    // For unit-style integration tests, we can test the router logic.
    
    #[worker_macros::test]
    async fn test_health_endpoint() {
        let router = router::build_router();
        let req = Request::new("https://example.com/api/health", Method::Get).unwrap();
        let env = Env::default(); // This might fail if it expects real bindings
        
        let resp = router.run(req, env).await.unwrap();
        assert_eq!(resp.status_code(), 200);
        
        let body: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(body["status"], "ok");
    }
}
