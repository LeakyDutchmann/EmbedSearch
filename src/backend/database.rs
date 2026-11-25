use std::path::Path;
use libsql::params;
use libsql::{Connection, Builder};
use std::collections::{HashSet, HashMap};

use crate::backend::embedding::DbIndexedImage;
use crate::backend::types::{ChatId, MessageId, TopicId};

pub struct Database {
    connection: Connection
}

impl Database {
    pub async fn initialize(database_path: impl AsRef<Path> ) -> anyhow::Result<Self> {
        let db = Builder::new_local(database_path).build().await?;
        
        let connection = db.connect()?;

        connection.execute("CREATE TABLE IF NOT EXISTS images (
            embedding BLOB NOT NULL,
            msg_id INT NOT NULL,
            chat_id INT NOT NULL,
            topic_id INT NULL,
            PRIMARY KEY (chat_id, msg_id)
        );", ()).await?;
        
        Ok(Database {
            connection
        })
    }
    pub async fn load_indexed(&self) -> anyhow::Result<Vec<DbIndexedImage>> {
        let mut rows = self.connection.query("SELECT msg_id, chat_id, embedding, topic_id FROM images", ()).await?;
        
        let mut result = Vec::new();
        let mut x = 0;
        while let Some(row) = rows.next().await? {
            x += 1;
            println!("Loaded {}", x);
            let msg_id: i32 = row.get(0)?;
            let chat_id: i64 = row.get(1)?;
            let bytes: Vec<u8> = row.get(2)?;    
            let topic_id: Option<i64> = row.get(3)?;

            let embedding: Vec<f32> = bytes
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes(
                    [chunk[0],chunk[1], chunk[2], chunk[3]]
                ))
                .collect();
            
            result.push(DbIndexedImage {
                msg_id: MessageId { id: msg_id },
                chat_id: ChatId{ id: chat_id},
                embedding: embedding.into_boxed_slice(),
                topic_id: topic_id.map(|id| TopicId{ id} ),
            });
        }

        Ok(result)
    }
    pub async fn get_indexed_scopes(&self) -> anyhow::Result<HashMap<ChatId, HashSet<TopicId>>>{
        let mut rows = self.connection.query("SELECT chat_id, topic_id FROM images", ()).await?;
        
        let mut map: HashMap<ChatId, HashSet<TopicId>> = HashMap::new();
        while let Some(row) = rows.next().await? {
            let id: i64 = row.get(0)?;
            let topic_id: Option<i64> = row.get(1)?;

            let e = map.entry(ChatId {id}).or_default();
            println!("Loaded id {}", id);
            if let Some(t_id) = topic_id {
                e.insert(TopicId { id: t_id});
            }
        }

        Ok(map)
    }
    pub async fn save_indexed(&mut self, indexed: &Vec<DbIndexedImage>) -> anyhow::Result<()> {
        let trans = self.connection.transaction().await?;

        let mut x = 0;
        for img in indexed {
            let bytes: &[u8] = bytemuck::cast_slice(
                img.embedding.as_ref()
            );
            let msg_id = img.msg_id.id;
            let chat_id = img.chat_id.id;
            
            let topic_id = {
                match &img.topic_id {
                    Some(t_id) => {
                        Some(t_id.id)
                    },
                    None => None
                }
            };
            
            trans.execute(
              "INSERT INTO images(msg_id, chat_id, embedding, topic_id) values(?1, ?2, ?3, ?4)",
              params![msg_id, chat_id, bytes, topic_id]
            ).await?;
            x += 1;
            println!("Inserted {} into db", x)
        }
        
        trans.commit().await?;
        Ok(())
    }
}
