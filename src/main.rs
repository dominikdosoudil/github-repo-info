use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use comfy_table::{Cell, Color, Table};
use octocrab;
use octocrab::params::repos::Type;

struct SumStats {
    stars: u32,
    forks: u32,
    followers: u32,
    updated_at: DateTime<Utc>,
    pushed_at: DateTime<Utc>,
    open_issues_count: u32,
    size: u32,
}

impl SumStats {
    pub fn new() -> Self {
        Self {
            stars: 0,
            forks: 0,
            followers: 0,
            updated_at: DateTime::<Utc>::from_utc(NaiveDateTime::MIN, Utc),
            pushed_at: DateTime::<Utc>::from_utc(NaiveDateTime::MIN, Utc),
            open_issues_count: 0,
            size: 0,
        }
    }

    pub fn update(
        &mut self,
        stars: u32,
        forks: u32,
        followers: u32,
        updated_at: DateTime<Utc>,
        pushed_at: DateTime<Utc>,
        open_issues_count: u32,
        size: u32,
    ) {
        self.stars += stars;
        self.forks += forks;
        self.followers += followers;
        self.updated_at = self.updated_at.max(updated_at);
        self.pushed_at = self.pushed_at.max(pushed_at);
        self.open_issues_count += open_issues_count;
        self.size += size;
    }
}

#[tokio::main]
async fn main() {
    let org_names: Vec<&str> = vec![];

    let github = octocrab::instance();

    for org_name in org_names {
        let org = github.orgs(org_name);
        let mut org_repos = org
            .list_repos()
            .repo_type(Type::Public)
            .send()
            .await
            .expect("find repos")
            .items;
        org_repos.sort_by(|a, b| b.pushed_at.unwrap().cmp(&a.pushed_at.unwrap()));

        let mut sum_stats = SumStats::new();
        let org_info = org.get().await.unwrap();
        let mut table = Table::new();
        table.add_row(vec![
            Cell::new("Repository").fg(Color::Green),
            Cell::new("Stars").fg(Color::Green),
            Cell::new("Forks").fg(Color::Green),
            Cell::new("License").fg(Color::Green),
            Cell::new("Followers").fg(Color::Green),
            Cell::new("Updated at").fg(Color::Green),
            Cell::new("Pushed at").fg(Color::Green),
            Cell::new("Open issues").fg(Color::Green),
            Cell::new("Size").fg(Color::Green),
            Cell::new("Created").fg(Color::Green),
        ]);
        for repo in org_repos.into_iter().take(5) {
            if repo.archived.unwrap() {
                continue;
            }
            let stars_n = repo.stargazers_count.unwrap();
            let forks_n = repo.forks_count.unwrap();
            table.add_row(vec![
                repo.name,
                stars_n.to_string(),
                forks_n.to_string(),
                repo.license.map(|l| l.name).unwrap_or("".to_string()),
                repo.watchers_count.unwrap().to_string(),
                repo.updated_at.unwrap().to_string(),
                repo.pushed_at.unwrap().to_string(),
                repo.open_issues_count.unwrap().to_string(),
                repo.size.unwrap().to_string(),
                repo.created_at.unwrap().year().to_string(),
            ]);
            sum_stats.update(
                stars_n,
                forks_n,
                repo.watchers_count.unwrap(),
                repo.updated_at.unwrap(),
                repo.pushed_at.unwrap(),
                repo.open_issues_count.unwrap(),
                repo.size.unwrap(),
            );
        }
        table.set_header(vec![
            Cell::new(format!(
                "{} [{}]",
                org_info.name.unwrap(),
                org_info.created_at.unwrap().year()
            ))
            .fg(Color::Green),
            Cell::new(format!("Sum: {}", sum_stats.stars)),
            Cell::new(format!("Sum: {}", sum_stats.forks)),
            Cell::new(""),
            Cell::new(format!("Sum: {}", sum_stats.followers)),
            Cell::new(format!("Latest: {}", sum_stats.updated_at)),
            Cell::new(format!("Latest: {}", sum_stats.pushed_at)),
            Cell::new(format!("Sum: {}", sum_stats.open_issues_count)),
            Cell::new(format!("Sum: {}", sum_stats.size)),
        ]);
        println!("{table}");
    }
}
