// Repository Implementations

pub mod alert_repository;
pub mod backup_repository;
pub mod homeassistant_repository;
pub mod security_repository;
pub mod weather_repository;

pub use alert_repository::{start_background_refresh, AlertRepositoryImpl};
pub use backup_repository::BackupRepositoryImpl;
pub use homeassistant_repository::{create_homeassistant_repository, HomeAssistantRepositoryImpl};
pub use security_repository::{create_security_repository, SecurityRepositoryImpl};
pub use weather_repository::{create_weather_repository, WeatherRepositoryImpl};

pub mod cloudflare_repository;
pub use cloudflare_repository::CloudflareRepositoryImpl;

pub mod cluster_repository;
pub use cluster_repository::KubernetesClusterRepository;


pub mod kubernetes;
pub mod a2ui_repository_impl;

pub use kubernetes::KubernetesRepositoryImpl;
pub use a2ui_repository_impl::A2UIRepositoryImpl;

pub mod s3_repository;
pub use s3_repository::S3TranscriptionRepository;

pub mod mqtt_notification_repository;
pub use mqtt_notification_repository::MqttNotificationRepository;

pub mod mock;
pub use mock::{MockClusterRepository, NoOpBackupRepository};
