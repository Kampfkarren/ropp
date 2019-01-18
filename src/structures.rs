use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fmt;

pub type PlaceId = u64;
pub type Result<T> = std::result::Result<T, RoppError>;
pub type Step = Vec<String>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoppError {
    EmptyStep,
    NoBuilds,
    PublishError(reqwest::StatusCode),
    RequestError(String),
    StepsUnspecifiedForBuild(String),
}

impl fmt::Display for RoppError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::RoppError::*;

        match self {
            EmptyStep => write!(formatter, "Step can't be empty"),
            NoBuilds => write!(formatter, "No builds listed in configuration."),
            PublishError(status_code) => write!(formatter, "Error while publishing, status code: {}\nThis usually means you don't have access to the place.", status_code),
            RequestError(error) => write!(formatter, "Request Error: {}", error),
            StepsUnspecifiedForBuild(build) => write!(formatter, "No steps listed for {}", build),
        }
    }
}

impl From<reqwest::Error> for RoppError {
    fn from(error: reqwest::Error) -> RoppError {
        RoppError::RequestError(error.to_string())
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub builds: HashMap<String, PlaceId>,
    pub steps: HashMap<String, Steps>,
}

impl Config {
    pub fn build_info(&self, build: &str) -> Option<(PlaceId, &Steps)> {
        if let Some(place_id) = self.builds.get(build) {
            if let Some(steps) = self.steps.get(build) {
                return Some((*place_id, steps));
            }
        }

        None
    }

    pub fn validate(&self) -> Result<()> {
        if self.builds.is_empty() {
            return Err(RoppError::NoBuilds);
        }

        for build in self.builds.keys() {
            match self.steps.get(build) {
                Some(steps) => {
                    for step in steps.pre.iter().chain(steps.post.iter()).flatten() {
                        if step.is_empty() {
                            return Err(RoppError::EmptyStep);
                        }
                    }
                }

                None => {
                    return Err(RoppError::StepsUnspecifiedForBuild(build.to_string()));
                }
            };
        }

        Ok(())
    }
}

#[derive(Debug, Default, Deserialize, Eq, PartialEq)]
pub struct Steps {
    pub pre: Option<Vec<Step>>,
    pub post: Option<Vec<Step>>,
}

#[cfg(test)]
mod tests {
    use super::{Config, RoppError, Steps};
    use std::collections::HashMap;

    macro_rules! map {
        () => {
            HashMap::new()
        };

        ($($key:expr => $value:expr,)+) => {{
            let mut map = HashMap::new();
            $(
                map.insert($key.into(), $value);
            )*
            map
        }};
    }

    #[test]
    fn test_config_validate() {
        assert_eq!(
            Config {
                builds: map!(),
                steps: map!(),
            }
            .validate(),
            Err(RoppError::NoBuilds),
        );

        assert_eq!(
            Config {
                builds: map!(
                    "a" => 1,
                ),
                steps: map!(),
            }
            .validate(),
            Err(RoppError::StepsUnspecifiedForBuild("a".to_string())),
        );

        assert_eq!(
            Config {
                builds: map!(
                    "a" => 1,
                ),
                steps: map!(
                    "a" => Steps {
                        pre: Some(vec![vec![]]),
                        post: None,
                    },
                ),
            }
            .validate(),
            Err(RoppError::EmptyStep)
        );

        assert_eq!(
            Config {
                builds: map!(
                    "a" => 1,
                ),
                steps: map!(
                    "a" => Steps::default(),
                ),
            }
            .validate(),
            Ok(()),
        );
    }

    #[test]
    fn test_config_build_info() {
        assert_eq!(
            Config {
                builds: map!(
                    "b" => 1,
                ),
                steps: map!(),
            }
            .build_info("a"),
            None,
        );

        assert_eq!(
            Config {
                builds: map!(
                    "a" => 1,
                ),
                steps: map!(),
            }
            .build_info("a"),
            None,
        );

        assert_eq!(
            Config {
                builds: map!(
                    "a" => 1,
                ),
                steps: map!(
                    "a" => Steps::default(),
                ),
            }
            .build_info("a"),
            Some((1, &Steps::default())),
        );
    }
}
