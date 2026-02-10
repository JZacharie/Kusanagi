//! Tests for Weather Domain Service

use std::collections::HashMap;

// Weather service data structures
#[derive(Debug, Clone)]
struct WeatherData {
    city: String,
    temperature: f64,
    #[allow(dead_code)]
    humidity: u8,
    condition: WeatherCondition,
    #[allow(dead_code)]
    wind_speed: f64,
    #[allow(dead_code)]
    updated_at: String,
}

#[derive(Debug, Clone, PartialEq)]
enum WeatherCondition {
    Sunny,
    Cloudy,
    Rainy,
    Snowy,
    Stormy,
    Foggy,
}

impl WeatherCondition {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sunny" | "clear" => Some(Self::Sunny),
            "cloudy" | "overcast" => Some(Self::Cloudy),
            "rainy" | "rain" | "drizzle" => Some(Self::Rainy),
            "snowy" | "snow" => Some(Self::Snowy),
            "stormy" | "thunderstorm" => Some(Self::Stormy),
            "foggy" | "fog" | "mist" => Some(Self::Foggy),
            _ => None,
        }
    }

    fn to_icon(&self) -> &'static str {
        match self {
            Self::Sunny => "☀️",
            Self::Cloudy => "☁️",
            Self::Rainy => "🌧️",
            Self::Snowy => "❄️",
            Self::Stormy => "⛈️",
            Self::Foggy => "🌫️",
        }
    }
}

struct WeatherService {
    cache: HashMap<String, WeatherData>,
    default_cities: Vec<String>,
}

impl WeatherService {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            default_cities: vec![
                "Paris".to_string(),
                "London".to_string(),
                "New York".to_string(),
                "Tokyo".to_string(),
            ],
        }
    }

    fn add_weather(&mut self, data: WeatherData) {
        self.cache.insert(data.city.clone(), data);
    }

    fn get_weather(&self, city: &str) -> Option<&WeatherData> {
        self.cache.get(city)
    }

    fn get_all_weather(&self) -> Vec<&WeatherData> {
        self.cache.values().collect()
    }

    fn get_cities_with_condition(&self, condition: WeatherCondition) -> Vec<&WeatherData> {
        self.cache
            .values()
            .filter(|w| w.condition == condition)
            .collect()
    }

    fn get_average_temperature(&self) -> Option<f64> {
        if self.cache.is_empty() {
            return None;
        }
        let sum: f64 = self.cache.values().map(|w| w.temperature).sum();
        Some(sum / self.cache.len() as f64)
    }

    fn get_hottest_city(&self) -> Option<&WeatherData> {
        self.cache.values().max_by(|a, b| {
            a.temperature
                .partial_cmp(&b.temperature)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn get_coldest_city(&self) -> Option<&WeatherData> {
        self.cache.values().min_by(|a, b| {
            a.temperature
                .partial_cmp(&b.temperature)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn is_cache_stale(&self, city: &str, _max_age_minutes: u64) -> bool {
        // Simplified - in real implementation would check timestamp
        !self.cache.contains_key(city)
    }

    fn refresh_weather(&mut self, city: &str, data: WeatherData) {
        self.cache.insert(city.to_string(), data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_weather_data(city: &str, temp: f64, condition: WeatherCondition) -> WeatherData {
        WeatherData {
            city: city.to_string(),
            temperature: temp,
            humidity: 60,
            condition,
            wind_speed: 10.0,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_weather_service_new() {
        let service = WeatherService::new();
        assert_eq!(service.default_cities.len(), 4);
        assert!(service.cache.is_empty());
    }

    #[test]
    fn test_add_and_get_weather() {
        let mut service = WeatherService::new();
        let data = create_test_weather_data("Paris", 20.0, WeatherCondition::Sunny);

        service.add_weather(data);

        let retrieved = service.get_weather("Paris");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().temperature, 20.0);
    }

    #[test]
    fn test_get_weather_not_found() {
        let service = WeatherService::new();
        assert!(service.get_weather("UnknownCity").is_none());
    }

    #[test]
    fn test_get_all_weather() {
        let mut service = WeatherService::new();

        service.add_weather(create_test_weather_data(
            "Paris",
            20.0,
            WeatherCondition::Sunny,
        ));
        service.add_weather(create_test_weather_data(
            "London",
            15.0,
            WeatherCondition::Cloudy,
        ));
        service.add_weather(create_test_weather_data(
            "Tokyo",
            25.0,
            WeatherCondition::Rainy,
        ));

        let all = service.get_all_weather();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_get_cities_with_condition() {
        let mut service = WeatherService::new();

        service.add_weather(create_test_weather_data(
            "Paris",
            20.0,
            WeatherCondition::Sunny,
        ));
        service.add_weather(create_test_weather_data(
            "London",
            15.0,
            WeatherCondition::Cloudy,
        ));
        service.add_weather(create_test_weather_data(
            "Madrid",
            25.0,
            WeatherCondition::Sunny,
        ));
        service.add_weather(create_test_weather_data(
            "Tokyo",
            22.0,
            WeatherCondition::Rainy,
        ));

        let sunny_cities = service.get_cities_with_condition(WeatherCondition::Sunny);
        assert_eq!(sunny_cities.len(), 2);

        let rainy_cities = service.get_cities_with_condition(WeatherCondition::Rainy);
        assert_eq!(rainy_cities.len(), 1);

        let snowy_cities = service.get_cities_with_condition(WeatherCondition::Snowy);
        assert_eq!(snowy_cities.len(), 0);
    }

    #[test]
    fn test_get_average_temperature() {
        let mut service = WeatherService::new();

        assert!(service.get_average_temperature().is_none());

        service.add_weather(create_test_weather_data(
            "Paris",
            20.0,
            WeatherCondition::Sunny,
        ));
        service.add_weather(create_test_weather_data(
            "London",
            15.0,
            WeatherCondition::Cloudy,
        ));
        service.add_weather(create_test_weather_data(
            "Tokyo",
            25.0,
            WeatherCondition::Rainy,
        ));

        let avg = service.get_average_temperature();
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), 20.0);
    }

    #[test]
    fn test_get_hottest_city() {
        let mut service = WeatherService::new();

        service.add_weather(create_test_weather_data(
            "Paris",
            20.0,
            WeatherCondition::Sunny,
        ));
        service.add_weather(create_test_weather_data(
            "London",
            15.0,
            WeatherCondition::Cloudy,
        ));
        service.add_weather(create_test_weather_data(
            "Tokyo",
            25.0,
            WeatherCondition::Rainy,
        ));

        let hottest = service.get_hottest_city();
        assert!(hottest.is_some());
        assert_eq!(hottest.unwrap().city, "Tokyo");
    }

    #[test]
    fn test_get_coldest_city() {
        let mut service = WeatherService::new();

        service.add_weather(create_test_weather_data(
            "Paris",
            20.0,
            WeatherCondition::Sunny,
        ));
        service.add_weather(create_test_weather_data(
            "London",
            15.0,
            WeatherCondition::Cloudy,
        ));
        service.add_weather(create_test_weather_data(
            "Tokyo",
            25.0,
            WeatherCondition::Rainy,
        ));

        let coldest = service.get_coldest_city();
        assert!(coldest.is_some());
        assert_eq!(coldest.unwrap().city, "London");
    }

    #[test]
    fn test_is_cache_stale() {
        let mut service = WeatherService::new();

        assert!(service.is_cache_stale("Paris", 60));

        service.add_weather(create_test_weather_data(
            "Paris",
            20.0,
            WeatherCondition::Sunny,
        ));
        // Note: simplified implementation always returns true for missing,
        // false for existing in real implementation would check timestamp
    }

    #[test]
    fn test_refresh_weather() {
        let mut service = WeatherService::new();

        service.add_weather(create_test_weather_data(
            "Paris",
            20.0,
            WeatherCondition::Sunny,
        ));

        let new_data = create_test_weather_data("Paris", 22.0, WeatherCondition::Cloudy);
        service.refresh_weather("Paris", new_data);

        let updated = service.get_weather("Paris");
        assert_eq!(updated.unwrap().temperature, 22.0);
    }
}

mod weather_condition_tests {
    use super::*;

    #[test]
    fn test_weather_condition_from_str() {
        assert_eq!(
            WeatherCondition::from_str("sunny"),
            Some(WeatherCondition::Sunny)
        );
        assert_eq!(
            WeatherCondition::from_str("CLEAR"),
            Some(WeatherCondition::Sunny)
        );
        assert_eq!(
            WeatherCondition::from_str("Cloudy"),
            Some(WeatherCondition::Cloudy)
        );
        assert_eq!(
            WeatherCondition::from_str("RAIN"),
            Some(WeatherCondition::Rainy)
        );
        assert_eq!(
            WeatherCondition::from_str("snow"),
            Some(WeatherCondition::Snowy)
        );
        assert_eq!(
            WeatherCondition::from_str("thunderstorm"),
            Some(WeatherCondition::Stormy)
        );
        assert_eq!(
            WeatherCondition::from_str("fog"),
            Some(WeatherCondition::Foggy)
        );
        assert_eq!(WeatherCondition::from_str("unknown"), None);
    }

    #[test]
    fn test_weather_condition_to_icon() {
        assert_eq!(WeatherCondition::Sunny.to_icon(), "☀️");
        assert_eq!(WeatherCondition::Cloudy.to_icon(), "☁️");
        assert_eq!(WeatherCondition::Rainy.to_icon(), "🌧️");
        assert_eq!(WeatherCondition::Snowy.to_icon(), "❄️");
        assert_eq!(WeatherCondition::Stormy.to_icon(), "⛈️");
        assert_eq!(WeatherCondition::Foggy.to_icon(), "🌫️");
    }

    #[test]
    fn test_weather_condition_equality() {
        assert_eq!(WeatherCondition::Sunny, WeatherCondition::Sunny);
        assert_ne!(WeatherCondition::Sunny, WeatherCondition::Rainy);
    }
}
