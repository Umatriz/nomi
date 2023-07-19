use std::path::PathBuf;

pub struct MavenData {
    pub url: String,
    pub url_file: String,
    pub local_file: String,
    pub local_file_path: PathBuf,
}

impl MavenData {
    pub fn new(maven_string: &str) -> Self {
        let maven = Self::unwrap_maven(maven_string);

        let mut maven_url = String::new();
        let mut local_path = PathBuf::new();
        maven.iter().for_each(|i| {
            maven_url.push('/');
            local_path = local_path.join(i);
            maven_url.push_str(&urlencoding::encode(i))
        });
        let maven_file = maven.iter().rev().collect::<Vec<_>>();

        Self {
            url: maven_url,
            url_file: format!(
                "{}-{}.jar",
                &urlencoding::encode(maven_file[1]),
                &urlencoding::encode(maven_file[0])
            ),
            local_file: format!("{}-{}.jar", maven_file[1], maven_file[0]),
            local_file_path: local_path,
        }
    }
    fn unwrap_maven(maven: &str) -> Vec<&str> {
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

        maven_vec
    }
}
