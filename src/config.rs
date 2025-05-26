use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,
    pub host: IpAddr,
    pub port: u16,
    pub database_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_name: "my_app".into(),
            host: Ipv4Addr::new(0, 0, 0, 0).into(),
            port: 3000,
            database_url: "".into(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        let default = Self::default();

        let app_name = env::var("APP_NAME").unwrap_or_default();
        let host = env::var("HOST")
            .unwrap_or_default()
            .parse::<IpAddr>()
            .unwrap_or(default.host);
        let port = env::var("PORT")
            .unwrap_or_default()
            .parse::<u16>()
            .unwrap_or_default();
        let database_url = env::var("DATABASE_URL").unwrap_or_default();

        Self {
            host,
            port,
            app_name,
            database_url,
        }
    }

    pub fn get_addr(&self) -> SocketAddr {
        SocketAddr::from((self.host, self.port))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        net::{IpAddr, Ipv4Addr},
    };

    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.app_name, "my_app");
        assert_eq!(config.port, 3000);
        assert_eq!(config.host, IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
    }

    #[test]
    fn test_config_with_env() {
        unsafe { env::set_var("APP_NAME", "test_app") };
        unsafe { env::set_var("HOST", "127.0.0.1") };

        let config = Config::new();
        assert_eq!(config.app_name, "test_app");
        assert_eq!(config.host, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[test]
    fn test_config_get_addr() {
        let config = Config::default();
        let addr = config.get_addr();
        assert_eq!(addr.port(), 3000);
        assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
    }
}
