#[cfg(test)]
mod tests {
    use kusanagi::domain::services::kubernetes_service;

    #[tokio::test]
    async fn test_parse_k8s_quantity() {
        let result = kubernetes_service::parse_k8s_quantity("1000m");
        assert_eq!(result, 1); // 1000 * 0.001 = 1

        let result = kubernetes_service::parse_k8s_quantity("500");
        assert_eq!(result, 500);

        let result = kubernetes_service::parse_k8s_quantity("2");
        assert_eq!(result, 2);

        let result = kubernetes_service::parse_k8s_quantity("1024Ki");
        assert_eq!(result, 1024 * 1024);

        let result = kubernetes_service::parse_k8s_quantity("1Mi");
        assert_eq!(result, 1024 * 1024);

        let result = kubernetes_service::parse_k8s_quantity("1.5Gi");
        assert_eq!(result, 1610612736);
    }

    #[tokio::test]
    async fn test_format_bytes() {
        let result = kubernetes_service::format_bytes(1024);
        assert_eq!(result, "1.00 KiB");

        let result = kubernetes_service::format_bytes(1048576);
        assert_eq!(result, "1.00 MiB");

        let result = kubernetes_service::format_bytes(1073741824);
        assert_eq!(result, "1.00 GiB");

        let result = kubernetes_service::format_bytes(0);
        assert_eq!(result, "0 B");
    }
}
