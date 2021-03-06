use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum RequestMethod {
    GET,
    POST,
    DELETE,
    PUT,
    PATCH,
    OPTIONS,
}

impl FromStr for RequestMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "GET" => RequestMethod::GET,
            "POST" => RequestMethod::POST,
            "DELETE" => RequestMethod::DELETE,
            "PUT" => RequestMethod::PUT,
            "PATCH" => RequestMethod::PATCH,
            "OPTIONS" => RequestMethod::OPTIONS,
            _ => Err(format!("unknown Request method '{}'", s.clone()))?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Request {
    method: RequestMethod,
    prefix: String,
    uri: String,
    pub controller: String,
    pub action: String,
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {}", self.method, self.uri)
    }
}

pub fn parse_routes(input: &str) -> Result<Vec<Request>, String> {
    if input.is_empty() {
        Err("input is empty".to_string())
    } else {
        let mut routes = Vec::new();
        println!("lines {}", input.lines().count());
        let foo: Vec<Vec<String>> = input
            .lines()
            .skip(1)
            .into_iter()
            .map(|f| {
                f.split_whitespace()
                    .map(|e| e.to_string())
                    .filter(|e| !e.is_empty())
                    .collect()
            })
            .collect();

        // this ugly mess is grabbing the valid feilds but ignoring the last one if an extra resource thing is added on to the end as I don't know what it does
        for i in 0..foo.len() {
            if foo[i].len() == 5 {
            } else if foo[i].len() == 4 {
                if let Ok(temp2) = RequestMethod::from_str(&foo[i][0]) {
                    let temp = foo[i][2].split("#").collect::<Vec<&str>>();
                    if temp.len() != 2 {
                        Err(format!(
                            "could not find action on the contorller {}",
                            foo[i][2]
                        ))?;
                    }

                    routes.push(Request {
                        method: temp2,
                        prefix: "".to_string(),
                        uri: foo[i][1].replace("(.:format)", ""),
                        controller: temp[0].to_string(),
                        action: temp[1].to_string(),
                    })
                } else {
                    let temp = foo[i][3].split("#").collect::<Vec<&str>>();
                    if temp.len() != 2 {
                        Err(format!(
                            "could not find action on the contorller {}",
                            foo[i][3]
                        ))?;
                    }

                    routes.push(Request {
                        method: RequestMethod::from_str(&foo[i][1])?,
                        prefix: foo[i][0].clone(),
                        uri: foo[i][2].replace("(.:format)", ""),
                        controller: temp[0].to_string(),
                        action: temp[1].to_string(),
                    })
                }
            } else if foo[i].len() == 3 {
                let temp = foo[i][2].split("#").collect::<Vec<&str>>();
                if temp.len() != 2 {
                    Err(format!(
                        "could not find action on the contorller {}",
                        foo[i][2]
                    ))?;
                }

                routes.push(Request {
                    method: RequestMethod::from_str(&foo[i][0])?,
                    prefix: "".to_string(),
                    uri: foo[i][1].replace("(.:format)", ""),
                    controller: temp[0].to_string(),
                    action: temp[1].to_string(),
                })
            } else {
                println!("panic {:?}", foo[i]);
            }
        }

        Ok(routes)
    }
}

#[cfg(test)]
mod routes_parsing {
    use super::parse_routes;
    use super::Request;
    use super::RequestMethod;

    #[test]
    fn parse() {
        let input = "Prefix Verb    URI Pattern                                                                              Controller#Action
        email_processor POST    /email_processor(.:format)                                                               griddler/emails#create
            dog_form GET     /dog/form(.:format)                                                                   dog_forms#show
                        PATCH   /dog/form(.:format)                                                                   dog_forms#update
                        PUT     /dog/form(.:format)                                                                   dog_forms#update
                        POST    /dog/form(.:format)                                                                   dog_forms#create
          dog_styles GET     /dogs/:dog_id/styles(.:format)                                                     dogs/styles#index
                        POST    /dogs/:dog_id/styles(.:format)                                                     dogs/styles#create
       new_dog_style GET     /dogs/:dog_id/styles/new(.:format)                                                 dogs/styles#new
        ";

        assert_eq!(parse_routes(input).is_ok(), true, "successfully parse");
        assert_eq!(parse_routes(input).unwrap().len(), 8);
        assert_eq!(
            parse_routes(input).unwrap()[0],
            Request {
                method: RequestMethod::POST,
                prefix: "email_processor".to_string(),
                uri: "/email_processor".to_string(),
                controller: "griddler/emails".to_string(),
                action: "create".to_string(),
            }
        );

        assert_eq!(
            parse_routes(input).unwrap()[2],
            Request {
                method: RequestMethod::PATCH,
                prefix: "".to_string(),
                uri: "/dog/form".to_string(),
                controller: "dog_forms".to_string(),
                action: "update".to_string(),
            }
        );
    }
}
