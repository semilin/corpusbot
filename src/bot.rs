use serde::{Deserialize, Serialize};
use serenity::{
    async_trait,
    client::Cache,
    http::client::Http,
    model::{
        channel::Message,
        gateway::Ready,
        id::{ChannelId, GuildId, UserId},
        prelude::{Channel, ChannelType, GuildChannel, MessageId},
        Timestamp,
    },
    prelude::*,
};
use std::{collections::HashMap, fs, path::Path, process::Command};

#[derive(Serialize, Deserialize)]
pub struct CorpusMessage {
    contents: String,
    author_name: String,
    author_id: UserId,
    message_id: MessageId,
    time: Timestamp,
}

impl CorpusMessage {
    pub fn from_message(m: &Message) -> CorpusMessage {
        CorpusMessage {
            contents: m.content.clone(),
            author_name: m.author.name.clone(),
            author_id: m.author.id,
            message_id: m.id,
            time: m.timestamp,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CorpusChannel {
    messages: Vec<CorpusMessage>,
    first_id: MessageId,
    last_id: MessageId,
}

impl CorpusChannel {
    pub fn new() -> Self {
        CorpusChannel {
            messages: vec![],
            first_id: MessageId(1),
            last_id: MessageId(1),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Corpus {
    channels: HashMap<String, CorpusChannel>,
    chunk: u64,
}

impl Corpus {
    pub fn new(chunk: u64) -> Corpus {
        Corpus {
            channels: HashMap::new(),
            chunk,
        }
    }
    pub fn write(&self) {
        let filename: String = self.chunk.to_string();
        let path = Path::new("chunks").join(filename);
        std::fs::write(path, serde_json::to_string_pretty(&self).unwrap()).unwrap();
    }
}

pub struct Bot {
    latest_corpus: Option<Corpus>,
    cache: Cache,
}

impl Bot {
    pub fn new(c: Corpus) -> Bot {
        Bot {
            latest_corpus: Some(c),
            cache: Cache::new(),
        }
    }
    pub fn new_corpus(&mut self) {
        match self.latest_corpus.as_mut() {
            Some(corpus) => {
                for channel in corpus.channels.values_mut() {
                    channel.messages = vec![];
                    channel.first_id = channel.last_id;
                }
                corpus.chunk += 1;
            }
            None => {
                self.latest_corpus = Some(Corpus {
                    channels: HashMap::new(),
                    chunk: 0,
                })
            }
        }
    }
    pub async fn add_to_corpus(&mut self, http: &Http) {
        println!("Starting corpus process");
        let channels = match GuildId(807843650717483049).channels(http).await {
            Ok(cnls) => cnls,
            Err(_) => panic!("can't get channels"),
        };
        let mut most_added = 0;
        loop {
            for channel in &channels {
                if channel.1.kind == ChannelType::Category {
                    continue;
                }
                let added = self.add_channel(http, channel.1).await;
                if added > most_added {
                    most_added = added;
                }
            }
            self.latest_corpus.as_ref().unwrap().write();
            let latest_corpus = self.latest_corpus.as_ref().unwrap();
            self.new_corpus();
            if most_added <= 10 {
                break;
            }
        }
    }
    pub async fn add_channel(&mut self, http: &Http, c: &GuildChannel) -> u64 {
        let channel_name: &str = &c.name;
        println!("{}", channel_name);
        let latest_corpus = self.latest_corpus.as_mut().unwrap();
        let channel = match latest_corpus.channels.get_mut(channel_name) {
            Some(channel) => channel,
            None => {
                latest_corpus
                    .channels
                    .insert(channel_name.to_string(), CorpusChannel::new());
                latest_corpus.channels.get_mut(channel_name).unwrap()
            }
        };
        let first_id = channel.first_id;
        let mut current_id = first_id;
        let mut messages_added = 0;
        while messages_added < 100 {
            let messages = c
                .messages(http, |retriever| retriever.after(current_id).limit(100))
                .await;
            let messages = match messages {
                Ok(msgs) => msgs,
                Err(e) => {
                    println!("{:?}", e);
                    break;
                }
            };
            if messages.len() == 0 {
                break;
            }
            for message in messages {
                channel.messages.push(CorpusMessage::from_message(&message));
                current_id = message.id;
                if &channel.last_id.0 < &current_id.0 {
                    channel.last_id = current_id;
                }
                messages_added += 1;
            }
        }
        println!("{}", current_id.created_at());
        messages_added
    }
}

pub struct Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        todo!();
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let mut bot = Bot::new(Corpus::new(0));
        let mut token = fs::read_to_string("token").expect("Error reading token file");
        token = token.trim().to_string();
        bot.add_to_corpus(&Http::new(&token)).await;
    }
}
