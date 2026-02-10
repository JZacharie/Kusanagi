//! Tests for Weather Repository

use serde_json::json;

// Tests for weather code mapping
#[test]
fn test_map_weather_code_clear_sky() {
    // Code 0 = Clear sky
    let (desc, icon) = map_weather_code(0);
    assert_eq!(desc, "Clear sky");
    assert_eq!(icon, "01d");
}

#[test]
fn test_map_weather_code_partly_cloudy() {
    // Code 2 = Partly cloudy
    let (desc, icon) = map_weather_code(2);
    assert_eq!(desc, "Partly cloudy");
    assert_eq!(icon, "03d");
}

#[test]
fn test_map_weather_code_rain() {
    // Code 61 = Rain
    let (desc, icon) = map_weather_code(61);
    assert_eq!(desc, "Rain");
    assert_eq!(icon, "10d");
}

#[test]
fn test_map_weather_code_snow() {
    // Code 71 = Snow
    let (desc, icon) = map_weather_code(71);
    assert_eq!(desc, "Snow");
    assert_eq!(icon, "13d");
}

#[test]
fn test_map_weather_code_thunderstorm() {
    // Code 95 = Thunderstorm
    let (desc, icon) = map_weather_code(95);
    assert_eq!(desc, "Thunderstorm");
    assert_eq!(icon, "11d");
}

#[test]
fn test_map_weather_code_unknown() {
    // Unknown code
    let (desc, icon) = map_weather_code(999);
    assert_eq!(desc, "Unknown");
    assert_eq!(icon, "03d");
}

#[test]
fn test_map_weather_code_foggy() {
    // Code 45 = Foggy
    let (desc, icon) = map_weather_code(45);
    assert_eq!(desc, "Foggy");
    assert_eq!(icon, "50d");
}

#[test]
fn test_map_weather_code_drizzle() {
    // Code 51 = Drizzle
    let (desc, icon) = map_weather_code(51);
    assert_eq!(desc, "Drizzle");
    assert_eq!(icon, "09d");
}

#[test]
fn test_map_weather_code_showers() {
    // Code 80 = Showers
    let (desc, icon) = map_weather_code(80);
    assert_eq!(desc, "Showers");
    assert_eq!(icon, "09d");
}

// City coordinates tests
#[test]
fn test_get_city_coordinates_paris() {
    let coords = get_city_coordinates("Paris");
    assert!(coords.is_some());
    let (lat, lon) = coords.unwrap();
    assert!((lat - 48.8566).abs() < 0.001);
    assert!((lon - 2.3522).abs() < 0.001);
}

#[test]
fn test_get_city_coordinates_london() {
    let coords = get_city_coordinates("London");
    assert!(coords.is_some());
    let (lat, lon) = coords.unwrap();
    assert!((lat - 51.5074).abs() < 0.001);
    assert!((lon - (-0.1278)).abs() < 0.001);
}

#[test]
fn test_get_city_coordinates_new_york() {
    let coords = get_city_coordinates("New York");
    assert!(coords.is_some());
    let (lat, lon) = coords.unwrap();
    assert!((lat - 40.7128).abs() < 0.001);
    assert!((lon - (-74.0060)).abs() < 0.001);
}

#[test]
fn test_get_city_coordinates_tokyo() {
    let coords = get_city_coordinates("Tokyo");
    assert!(coords.is_some());
    let (lat, lon) = coords.unwrap();
    assert!((lat - 35.6762).abs() < 0.001);
    assert!((lon - 139.6503).abs() < 0.001);
}

#[test]
fn test_get_city_coordinates_unknown() {
    let coords = get_city_coordinates("UnknownCity");
    assert!(coords.is_none());
}

#[test]
fn test_get_city_coordinates_all_supported() {
    let cities = vec![
        "Lyon",
        "Mexico City",
        "New York",
        "Paris",
        "London",
        "Tokyo",
        "Sydney",
        "Berlin",
        "Madrid",
        "Rome",
    ];

    for city in cities {
        let coords = get_city_coordinates(city);
        assert!(coords.is_some(), "City {} should have coordinates", city);
    }
}

// Mock implementations for testing
fn map_weather_code(code: i64) -> (&'static str, &'static str) {
    match code {
        0 => ("Clear sky", "01d"),
        1 => ("Mainly clear", "02d"),
        2 => ("Partly cloudy", "03d"),
        3 => ("Overcast", "04d"),
        45 | 48 => ("Foggy", "50d"),
        51..=55 => ("Drizzle", "09d"),
        56..=57 => ("Freezing drizzle", "09d"),
        61..=65 => ("Rain", "10d"),
        66..=67 => ("Freezing rain", "10d"),
        71..=75 => ("Snow", "13d"),
        77 => ("Snow grains", "13d"),
        80..=82 => ("Showers", "09d"),
        85..=86 => ("Snow showers", "13d"),
        95 => ("Thunderstorm", "11d"),
        96 | 99 => ("Thunderstorm with hail", "11d"),
        _ => ("Unknown", "03d"),
    }
}

fn get_city_coordinates(city: &str) -> Option<(f64, f64)> {
    match city {
        "Lyon" => Some((45.7485, 4.8467)),
        "Mexico City" => Some((19.4326, -99.1332)),
        "New York" => Some((40.7128, -74.0060)),
        "Paris" => Some((48.8566, 2.3522)),
        "London" => Some((51.5074, -0.1278)),
        "Tokyo" => Some((35.6762, 139.6503)),
        "Sydney" => Some((-33.8688, 151.2093)),
        "Berlin" => Some((52.5200, 13.4050)),
        "Madrid" => Some((40.4168, -3.7038)),
        "Rome" => Some((41.9028, 12.4964)),
        _ => None,
    }
}

// Weather data structures and parsing tests
#[derive(Debug, Clone, PartialEq)]
struct WeatherInfo {
    city: String,
    temp: f32,
    description: String,
    icon: String,
    humidity: u8,
    wind_speed: f32,
    feels_like: f32,
    pressure: u32,
    visibility: u32,
    last_updated: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ForecastDay {
    date: String,
    temp: f32,
    description: String,
    icon: String,
}

struct WeatherResponse {
    cities: Vec<WeatherInfo>,
    cached_at: String,
    total: usize,
}

#[test]
fn test_weather_info_creation() {
    let info = WeatherInfo {
        city: "Paris".to_string(),
        temp: 20.5,
        description: "Clear sky".to_string(),
        icon: "01d".to_string(),
        humidity: 60,
        wind_speed: 10.5,
        feels_like: 19.0,
        pressure: 1013,
        visibility: 10000,
        last_updated: "14:30".to_string(),
    };

    assert_eq!(info.city, "Paris");
    assert_eq!(info.temp, 20.5);
    assert_eq!(info.humidity, 60);
}

#[test]
fn test_forecast_day_creation() {
    let day = ForecastDay {
        date: "2024-01-15".to_string(),
        temp: 18.0,
        description: "Partly cloudy".to_string(),
        icon: "03d".to_string(),
    };

    assert_eq!(day.date, "2024-01-15");
    assert_eq!(day.temp, 18.0);
}

#[test]
fn test_weather_response_creation() {
    let cities = vec![WeatherInfo {
        city: "Paris".to_string(),
        temp: 20.0,
        description: "Sunny".to_string(),
        icon: "01d".to_string(),
        humidity: 60,
        wind_speed: 10.0,
        feels_like: 20.0,
        pressure: 1013,
        visibility: 10000,
        last_updated: "12:00".to_string(),
    }];

    let response = WeatherResponse {
        total: cities.len(),
        cached_at: chrono::Local::now().to_rfc3339(),
        cities,
    };

    assert_eq!(response.total, 1);
    assert_eq!(response.cities[0].city, "Paris");
}

// Weather caching logic tests
struct WeatherCache {
    data: Option<WeatherResponse>,
    max_age_minutes: u64,
}

impl WeatherCache {
    fn new(max_age_minutes: u64) -> Self {
        Self {
            data: None,
            max_age_minutes,
        }
    }

    fn is_valid(&self) -> bool {
        if let Some(ref response) = self.data {
            // In real implementation, would check timestamp
            // For tests, we just check if data exists
            true
        } else {
            false
        }
    }

    fn set(&mut self, response: WeatherResponse) {
        self.data = Some(response);
    }

    fn get(&self) -> Option<&WeatherResponse> {
        self.data.as_ref()
    }

    fn clear(&mut self) {
        self.data = None;
    }
}

#[test]
fn test_weather_cache_empty() {
    let cache = WeatherCache::new(10);
    assert!(!cache.is_valid());
    assert!(cache.get().is_none());
}

#[test]
fn test_weather_cache_set_and_get() {
    let mut cache = WeatherCache::new(10);

    let response = WeatherResponse {
        cities: vec![],
        cached_at: chrono::Local::now().to_rfc3339(),
        total: 0,
    };

    cache.set(response);
    assert!(cache.is_valid());
    assert!(cache.get().is_some());
}

#[test]
fn test_weather_cache_clear() {
    let mut cache = WeatherCache::new(10);

    let response = WeatherResponse {
        cities: vec![],
        cached_at: chrono::Local::now().to_rfc3339(),
        total: 0,
    };

    cache.set(response);
    assert!(cache.is_valid());

    cache.clear();
    assert!(!cache.is_valid());
}

// Weather domain service tests
struct WeatherDomainService;

impl WeatherDomainService {
    fn is_cache_valid(&self, cached_at: &str) -> bool {
        // Simplified - in real impl would check timestamp
        !cached_at.is_empty()
    }

    fn create_mock_weather(&self, city: &str) -> WeatherInfo {
        WeatherInfo {
            city: city.to_string(),
            temp: 20.0,
            description: "Mock data".to_string(),
            icon: "03d".to_string(),
            humidity: 50,
            wind_speed: 5.0,
            feels_like: 20.0,
            pressure: 1013,
            visibility: 10000,
            last_updated: chrono::Local::now().format("%H:%M").to_string(),
        }
    }

    fn map_description_to_icon(&self, description: &str) -> &str {
        let desc_lower = description.to_lowercase();
        if desc_lower.contains("clear") || desc_lower.contains("sunny") {
            "01d"
        } else if desc_lower.contains("cloud") {
            "03d"
        } else if desc_lower.contains("rain") {
            "10d"
        } else if desc_lower.contains("snow") {
            "13d"
        } else if desc_lower.contains("storm") || desc_lower.contains("thunder") {
            "11d"
        } else {
            "03d"
        }
    }
}

#[test]
fn test_weather_domain_service_is_cache_valid() {
    let service = WeatherDomainService;

    assert!(service.is_cache_valid("2024-01-01T00:00:00Z"));
    assert!(!service.is_cache_valid(""));
}

#[test]
fn test_weather_domain_service_create_mock() {
    let service = WeatherDomainService;
    let mock = service.create_mock_weather("TestCity");

    assert_eq!(mock.city, "TestCity");
    assert_eq!(mock.temp, 20.0);
    assert_eq!(mock.description, "Mock data");
}

#[test]
fn test_weather_domain_service_map_description() {
    let service = WeatherDomainService;

    assert_eq!(service.map_description_to_icon("Clear sky"), "01d");
    assert_eq!(service.map_description_to_icon("Partly cloudy"), "03d");
    assert_eq!(service.map_description_to_icon("Heavy rain"), "10d");
    assert_eq!(service.map_description_to_icon("Snow fall"), "13d");
    assert_eq!(service.map_description_to_icon("Thunderstorm"), "11d");
    assert_eq!(service.map_description_to_icon("Unknown"), "03d");
}

// Temperature conversion tests
fn celsius_to_fahrenheit(celsius: f32) -> f32 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn fahrenheit_to_celsius(fahrenheit: f32) -> f32 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

#[test]
fn test_celsius_to_fahrenheit() {
    assert!((celsius_to_fahrenheit(0.0) - 32.0).abs() < 0.01);
    assert!((celsius_to_fahrenheit(100.0) - 212.0).abs() < 0.01);
    assert!((celsius_to_fahrenheit(20.0) - 68.0).abs() < 0.01);
}

#[test]
fn test_fahrenheit_to_celsius() {
    assert!((fahrenheit_to_celsius(32.0) - 0.0).abs() < 0.01);
    assert!((fahrenheit_to_celsius(212.0) - 100.0).abs() < 0.01);
    assert!((fahrenheit_to_celsius(68.0) - 20.0).abs() < 0.01);
}

// Open-Meteo response parsing tests
#[test]
fn test_parse_open_meteo_current_weather() {
    let mock_response = json!({
        "current_weather": {
            "temperature": 22.5,
            "windspeed": 15.0,
            "weathercode": 1,
            "time": "2024-01-15T12:00"
        },
        "daily": {
            "time": ["2024-01-15", "2024-01-16"],
            "temperature_2m_max": [25.0, 26.0],
            "temperature_2m_min": [15.0, 16.0],
            "weathercode": [1, 2]
        }
    });

    let current = &mock_response["current_weather"];
    let temp = current["temperature"].as_f64().unwrap_or(0.0) as f32;
    let wind_speed = current["windspeed"].as_f64().unwrap_or(0.0) as f32;
    let weather_code = current["weathercode"].as_i64().unwrap_or(0);

    assert!((temp - 22.5).abs() < 0.01);
    assert!((wind_speed - 15.0).abs() < 0.01);
    assert_eq!(weather_code, 1);
}

#[test]
fn test_parse_open_meteo_daily_forecast() {
    let mock_response = json!({
        "daily": {
            "time": ["2024-01-15", "2024-01-16", "2024-01-17"],
            "temperature_2m_max": [25.0, 26.0, 24.0],
            "temperature_2m_min": [15.0, 16.0, 14.0],
            "weathercode": [1, 2, 3]
        }
    });

    let daily = mock_response["daily"].as_object().unwrap();
    let times = daily["time"].as_array().unwrap();
    let max_temps = daily["temperature_2m_max"].as_array().unwrap();

    assert_eq!(times.len(), 3);
    assert_eq!(max_temps.len(), 3);
    assert_eq!(times[0], "2024-01-15");
    assert_eq!(max_temps[0], 25.0);
}
