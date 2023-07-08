pub struct MavenUrl(String, String);

impl MavenUrl {
    pub fn build_url(maven: Vec<&str>) -> Self {
        let mut maven_url = String::new();
        maven.iter().for_each(|i| {
            maven_url.push('/');
            maven_url.push_str(&urlencoding::encode(i))
        });
        let maven_file = maven.iter().rev().collect::<Vec<_>>();
        MavenUrl(
            maven_url,
            format!(
                "{}-{}",
                &urlencoding::encode(maven_file[1]),
                &urlencoding::encode(maven_file[0])
            ),
        )
    }
    pub fn unwrap_maven(maven: &str) -> Vec<&str> {
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
