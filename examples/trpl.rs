use std::env::args;
use trpl::Html;

fn main() {
    let args: Vec<String> = args().collect();
    let url = &args[1];
    trpl::run(async {
        match page_title(url).await {
            Some(title) => println!("The title for {} was {}", url, title),
            None => println!("{} had no title", url),
        }
    })
}

async fn page_title(url: &str) -> Option<String> {
    let response_text = trpl::get(url).await.text().await;
    Html::parse(&response_text)
        .select_first("title")
        .map(|title_element| title_element.inner_html())
}
