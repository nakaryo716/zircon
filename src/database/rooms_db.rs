use crate::models::rooms_model::{Room, RoomInfo};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use tokio::sync::broadcast;
use uuid::Uuid;

// ユーザーが開設したチャットルームのメタ情報及びtokio::bradcastのチャンネルを保持して使えるようにする構造体
pub struct RoomDb {
    pool: Arc<RwLock<HashMap<String, Room>>>,
}

impl RoomDb {
    pub fn new() -> Self {
        Self {
            pool: Arc::default(),
        }
    }
}

// RoomDbに保持されているデータ管理を行うTrait
pub trait RoomManage {
    type Info;
    type Data;
    type Error;
    fn open_new_room(
        &self,
        room_name: &str,
        created_by_id: &str,
    ) -> Result<Self::Info, Self::Error>;
    fn listen_room(&self, room_id: &str) -> Result<Self::Data, Self::Error>;
    fn delete_room(&self, room_id: &str) -> Result<(), Self::Error>;
}

impl RoomManage for RoomDb {
    type Info = RoomInfo;
    type Data = Room;
    type Error = RoomError;
    fn open_new_room(
        &self,
        room_name: &str,
        created_by_id: &str,
    ) -> Result<Self::Info, Self::Error> {
        let room = init_room(room_name, created_by_id);
        let mut gurad = get_write_lock(&self).map_err(|e| e)?;
        gurad.insert(room.get_room_info().get_room_id().to_string(), room.clone());

        Ok(room.get_room_info().to_owned())
    }

    // ルーム作成者以外の人がチャットルームに参加するためのメソッド
    fn listen_room(&self, room_id: &str) -> Result<Self::Data, Self::Error> {
        let room = get_read_lock(&self).and_then(|guard| {
            guard
                .get(room_id)
                .map(|e| e.to_owned())
                .ok_or_else(|| RoomError::IdNotFound)
        })?;
        Ok(room)
    }

    // ルームを削除するメソッド
    fn delete_room(&self, room_id: &str) -> Result<(), Self::Error> {
        let _ = get_write_lock(&self)
            .and_then(|mut gurad| gurad.remove(room_id).ok_or_else(|| RoomError::IdNotFound));
        Ok(())
    }
}

// ユニークIDを割り振る
// チャンネルの作成を行う
fn init_room(room_name: &str, created_by_id: &str) -> Room {
    let (sender, _) = broadcast::channel(128);

    Room {
        room_info: RoomInfo {
            room_id: Uuid::new_v4().to_string(),
            room_name: room_name.to_owned(),
            created_by: created_by_id.to_owned(),
        },
        sender,
    }
}

fn get_write_lock(db: &RoomDb) -> Result<RwLockWriteGuard<HashMap<String, Room>>, RoomError> {
    let lock = db.pool.write().map_err(|_| RoomError::DbError)?;
    Ok(lock)
}

fn get_read_lock(db: &RoomDb) -> Result<RwLockReadGuard<HashMap<String, Room>>, RoomError> {
    let lock = db.pool.read().map_err(|_| RoomError::DbError)?;
    Ok(lock)
}

pub enum RoomError {
    DbError,
    IdNotFound,
}
