use crate::data::{Day, Group, Repeat, Schedule, Slot, Subject};
use sqlx::SqlitePool as Pool;
use teloxide::types::ChatId;

pub struct Database {
    pool: Pool,
}

impl Database {
    pub fn new(pool: Pool) -> Database {
        Database { pool }
    }

    pub async fn add_subject(&self, value: &Subject) -> sqlx::Result<()> {
        let Subject {
            id,
            title,
            group,
            optional,
        } = value;
        let gang: String = group.into();
        sqlx::query!(
            "INSERT INTO subjects(id, title, gang, optional) VALUES(?, ?, ?, ?);",
            id,
            title,
            gang,
            optional
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn add_schedule(&self, value: &Schedule) -> sqlx::Result<()> {
        let Schedule {
            subject_id,
            day,
            repeat,
            slot,
        } = value;
        let day = *day as u8;
        let repeat = *repeat as u8;
        let slot = *slot as u8;

        sqlx::query!(
            "INSERT INTO schedule(day, repeat, slot, subject_id) VALUES(?, ?, ?, ?);",
            day,
            repeat,
            slot,
            subject_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_subjects(
        &self,
        day: Day,
        repeat: Repeat,
        slot: Slot,
        group: Group,
    ) -> sqlx::Result<Vec<Subject>> {
        let gang = String::from(&group);
        let repeat = repeat as u8;
        let slot = slot as u8;
        let day = day as u8;

        let records = sqlx::query!("SELECT * FROM subjects WHERE id = (SELECT subject_id FROM schedule WHERE day = ? AND (repeat = ? OR repeat = ?) AND slot = ?) AND gang = ?;", day, repeat, Repeat::Both as u8, slot, gang).fetch_all(&self.pool).await?;
        let mut subjects = Vec::with_capacity(records.len());
        for record in records {
            let (id, title, group, optional) = (
                record.id,
                record.title,
                Group::try_from(record.gang.as_str()).unwrap(),
                record.optional == 1,
            );
            let subject = Subject {
                id,
                title,
                group,
                optional,
            };
            subjects.push(subject);
        }
        Ok(subjects)
    }

    pub async fn add_user(&self, id: &ChatId, group: &Group) -> sqlx::Result<()> {
        let gang = String::from(group);
        sqlx::query!("INSERT INTO users(chat_id, gang) VALUES(?, ?);", id.0, gang)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_user(&self, id: &ChatId, group: &Group) -> sqlx::Result<()> {
        let gang = String::from(group);
        sqlx::query!("UPDATE users SET gang = ? WHERE chat_id = ?;", gang, id.0)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_group(&self, id: &ChatId) -> sqlx::Result<Group> {
        let rec = sqlx::query!("SELECT gang FROM users WHERE chat_id = ?;", id.0)
            .fetch_one(&self.pool)
            .await?;
        Ok(Group::try_from(rec.gang.as_str()).unwrap())
    }
}
