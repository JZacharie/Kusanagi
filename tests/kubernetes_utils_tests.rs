#[cfg(test)]
mod tests {
    use kusanagi::domain::services::kubernetes_service;

    #[tokio::test]
    async fn test_parse_k8s_quantity_millicores() {
        assert_eq!(kubernetes_service::parse_k8s_quantity("1000m"), 0);
        assert_eq!(kubernetes_service::parse_k8s_quantity("500m"), 0);
    }

    #[tokio::test]
    async fn test_parse_k8s_quantity_plain() {
        assert_eq!(kubernetes_service::parse_k8s_quantity("2"), 2);
        assert_eq!(kubernetes_service::parse_k8s_quantity("10"), 10);
    }

    #[tokio::test]
    async fn test_parse_k8s_quantity_ki() {
        assert_eq!(kubernetes_service::parse_k8s_quantity("1Ki"), 1024);
        assert_eq!(kubernetes_service::parse_k8s_quantity("2Ki"), 2048);
    }

    #[tokio::test]
    async fn test_parse_k8s_quantity_mi() {
        assert_eq!(kubernetes_service::parse_k8s_quantity("1Mi"), 1024 * 1024);
        assert_eq!(kubernetes_service::parse_k8s_quantity("2Mi"), 2 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_parse_k8s_quantity_gi() {
        assert_eq!(kubernetes_service::parse_k8s_quantity("1Gi"), 1024 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_parse_k8s_quantity_empty() {
        assert_eq!(kubernetes_service::parse_k8s_quantity(""), 0);
        assert_eq!(kubernetes_service::parse_k8s_quantity("  "), 0);
    }

    #[tokio::test]
    async fn test_format_bytes_zero() {
        assert_eq!(kubernetes_service::format_bytes(0), "0 B");
    }

    #[tokio::test]
    async fn test_format_bytes_kib() {
        assert_eq!(kubernetes_service::format_bytes(1024), "1.0 KiB");
        assert_eq!(kubernetes_service::format_bytes(2048), "2.0 KiB");
    }

    #[tokio::test]
    async fn test_format_bytes_mib() {
        assert_eq!(kubernetes_service::format_bytes(1048576), "1.0 MiB");
        assert_eq!(kubernetes_service::format_bytes(2097152), "2.0 MiB");
    }

    #[tokio::test]
    async fn test_format_bytes_gib() {
        assert_eq!(kubernetes_service::format_bytes(1073741824), "1.0 GiB");
    }

    #[tokio::test]
    async fn test_format_bytes_fractional() {
        let result = kubernetes_service::format_bytes(1536);
        assert!(result.starts_with("1.5"));
        assert!(result.ends_with("KiB"));
    }
}
