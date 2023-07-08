use reqwest::Client;

use super::fabric_meta::Meta;

pub struct FabricLoader {
    pub meta: Meta,
}

impl FabricLoader {
    pub async fn get_json() -> anyhow::Result<Meta> {
        let response: Meta = Client::new()
            .get("https://meta.fabricmc.net/v2/versions/loader/1.18.2")
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }
    pub async fn unwrap_maven() -> anyhow::Result<()> {
        let maven = "net.fabricmc:tiny-mappings-parser:0.3.0+build.17";
        let splited = maven.split(':').rev().collect::<Vec<_>>();
        let first_element = splited[0];
        let res = splited
            .iter()
            .map(|i| {
                if i == &first_element {
                    vec![i as &str]
                } else {
                    i.split('.').collect::<Vec<&str>>()
                }
            })
            .rev()
            .collect::<Vec<_>>();
        let mut maven_vec: Vec<&str> = vec![];
        res.iter()
            .for_each(|i| i.iter().for_each(|j| maven_vec.push(j)));
        println!("{:#?}", maven_vec);
        Ok(())
    }
}
