use regex::Regex;
use scraper::{Html, Selector};
use teloxide::{
    prelude::*,
    types::{
        InlineQueryResult, InlineQueryResultArticle, InputMessageContent, InputMessageContentText,
    },
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();
    let handler = Update::filter_inline_query().branch(dptree::endpoint(
        |bot: Bot, query: InlineQuery| async move {
            if let Ok(response) = look_up(&query.query).await {
                if let Some(username) = query.from.username {
                    log::info!("{}: {}", username, query.query);
                } else {
                    log::info!("{}: {}", query.from.id, query.query);
                }
                let result = InlineQueryResultArticle::new(
                    &query.id,
                    "搜尋法條",
                    InputMessageContent::Text(InputMessageContentText::new(response)),
                );
                let result = vec![InlineQueryResult::Article(result)];
                let res = bot.answer_inline_query(&query.id, result).send().await;
                if let Err(e) = res {
                    log::error!("{:?}", e);
                }
                return respond(());
            }
            respond(())
        },
    ));
    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn look_up(directive: &str) -> Result<String, reqwest::Error> {
    let directive = directive.trim().split(' ').collect::<Vec<&str>>();
    if directive.len() != 2 {
        return Ok("非法請求".into());
    }
    let pcode = find_pcode(directive[0]).await?;

    let url = format!(
        "https://law.moj.gov.tw/LawClass/LawSingle.aspx?{}&flno={}",
        pcode, directive[1]
    );
    dbg!(&url);
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let doc = Html::parse_fragment(&body);
    let article_selector = Selector::parse(".law-article").unwrap();
    if let Some(article) = doc.select(&article_selector).next() {
        let law_name_selector = Selector::parse("a").unwrap();
        let law_name = doc
            .select(&law_name_selector)
            .find(|x| x.value().id() == Some("hlLawName"))
            .unwrap()
            .inner_html();
        let result = article.text().collect::<Vec<&str>>().join("\n");
        let result = format!("{} 第 {} 條 {}", law_name, directive[1], result);
        log::info!("Response: {} 第 {} 條", law_name, directive[1]);
        Ok(result)
    } else {
        Ok("無此資料".into())
    }
}

async fn find_pcode(law_name: &str) -> Result<String, reqwest::Error> {
    let url = format!(
        "https://law.moj.gov.tw/Law/LawSearchResult.aspx?ty=ONEBAR&kw={}",
        law_name
    );
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let doc = Html::parse_fragment(&body);
    let selector = Selector::parse("a").unwrap();
    if let Some(first_res) = doc
        .select(&selector)
        .find(|x| x.value().id() == Some("hlkLawLink"))
    {
        let first_res = first_res.value().attr("href").unwrap();
        dbg!(&first_res);
        let re = Regex::new(r"pcode=([^&]+)").unwrap();
        let result = re
            .captures(first_res)
            .unwrap()
            .get(0)
            .unwrap()
            .as_str()
            .into();

        return Ok(result);
    }
    Ok("崩囉!".into())
}
