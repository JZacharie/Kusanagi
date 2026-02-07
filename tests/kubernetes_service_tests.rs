#[cfg(test)]
mod tests {
    use kusanagi::domain::services::kubernetes_service;
    use serde_json::json;

    #[tokio::test]
    async fn test_parse_k8s_quantity() {
        let result = kubernetes_service::parse_k8s_quantity("1000m");
        assert_eq!(result, 1.0);
        
        let result = kubernetes_service::parse_k8s_quantity("500m");
        assert_eq!(result, 0.5);
        
        let result = kubernetes_service::parse_k8s_quantity("2");
        assert_eq!(result, 2.0);
    }

    #[tokio::test]
    async fn test_format_bytes() {
        let result = kubernetes_service::format_bytes(1024);
        assert_eq!(result, "1.00 KB");
        
        let result = kubernetes_service::format_bytes(1048576);
        assert_eq!(result, "1.00 MB");
        
        let result = kubernetes_service::format_bytes(1073741824);
        assert_eq!(result, "1.00 GB");
    }

    #[tokio::test]
    async fn test_get_cluster_overview_fallback() {
        // Test that it returns valid JSON even without k8s cluster
        let result = kubernetes_service::get_cluster_overview().await;
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert!(data["nodes"].as_u64().is_some());
        assert!(data["pods"].as_u64().is_some());
        assert!(data["services"].as_u64().is_some());
    }
}
