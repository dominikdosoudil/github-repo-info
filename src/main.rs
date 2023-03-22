use chrono::{DateTime, Datelike, NaiveDateTime, Utc};

use clap::{arg, Parser};
use comfy_table::{Cell, Color, ContentArrangement, Table};
use octocrab;
use octocrab::params::repos::Type;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Parser, Debug)]
struct Cli {
    orgs: Vec<String>,

    #[arg(
        short,
        long,
        value_name = "Take only n latest repositories for compactness",
        default_value_t = usize::MAX
    )]
    latest_n: usize,
}

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
async fn main() -> Result<(), String> {
    let args = Cli::parse();

    let github = octocrab::instance();

    let mut out_file = File::create("out/org_stats.csv")
        .await
        .expect("open csv file ok");

    out_file.write_all(b"real_org_name,org_created_at,stars,forks,followers,updated_at,pushed_at,open_issues_count,size\n").await.expect("csv file write ok");

    for org_name in args.orgs {
        let org = github.orgs(&org_name);
        match org.get().await {
            Ok(org_info) => {
                let mut org_repos = org
                    .list_repos()
                    .repo_type(Type::Public)
                    .send()
                    .await
                    .expect("find repos")
                    .items;
                org_repos.sort_by(|a, b| b.pushed_at.unwrap().cmp(&a.pushed_at.unwrap()));

                let mut sum_stats = SumStats::new();
                let mut table = Table::new();
                table.set_content_arrangement(ContentArrangement::DynamicFullWidth);
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
                for repo in org_repos.into_iter().take(args.latest_n) {
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
                let real_org_name = org_info.name.unwrap_or(org_name);
                let org_created_at = org_info.created_at.unwrap().year();
                table.set_header(vec![
                    Cell::new(format!("{} [{}]", real_org_name, org_created_at,)).fg(Color::Green),
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
                out_file
                    .write_all(
                        format!(
                            "{},{},{},{},{},{},{},{},{}\n",
                            real_org_name,
                            org_created_at,
                            sum_stats.stars,
                            sum_stats.forks,
                            sum_stats.followers,
                            sum_stats.updated_at,
                            sum_stats.pushed_at,
                            sum_stats.open_issues_count,
                            sum_stats.size
                        )
                        .as_bytes(),
                    )
                    .await
                    .expect("write csv row ok");
            }
            Err(e) => {
                println!("Organization {org_name} not found {e}");
            }
        }
    }
    Ok(())
}
