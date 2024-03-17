use sea_orm::*;


#[derive(Debug)]

pub struct ClickDao;

impl ClickDao {
    //创建一个点击记录
    async fn create_click(
        db:&DatabaseConnection,
        level_id:i32,
        date:NaiveDate,
    ) -> Result<((),DbErr)> {
        let mut tx: sea_orm::DatabaseTransaction = db.begin().await.map_err(SystemError::DbErr)?;
        Self::do_create(&mut tx,level_id, date).await?;
        if commit {
            tx.commit().await.map_err(SystemError::DbErr)?;
        }
        return Ok(());
    }
    //创建点击记录但是不提交具体事务
    async fn do_create(
        tx:&TransactionConnection,
        id_click:i32,
        date:NaiveDate,
    ) -> Result<((),DbErr)> {
        let click = click::ActiveModel {
            level_id: ActiceValue::Set(id_click),
            count_today : ActiceValue::Set(1),
            date: ActiceValue::Set(date),
            ..Default::default()
        };
        click.save(tx).await.map_err(SystemError::DbErr)?;

        return Ok(());
    }
    
    pub async fn get_today_click_count(
        app_data: &web::Data<AppState>,
        level_id: i32,
        with_fake: bool,
    ) -> Result<i32, SystemError> {
        let today = chrono::Utc::now().naive_utc().date();
        Self::get_click_count(app_data, mood_id, today, with_fake).await
    }
    //获取点击次数
    pub async fn get_click_count(
        app_data: &web::Data<AppState>,
        level_id: i32,
        date: NaiveDate,
        with_fake: bool,
    ) -> Result<i32, SystemError> {
        let db = app_data.db_conn();

        let mut r = crate::entity::click::Entity::find()
            .filter(crate::entity::click::Column::IdMood.eq(level_id))
            .filter(crate::entity::click::Column::Date.eq(date))
            .one(db)
            .await
            .map_err(|e| {
                log::error!("Failed to get click count: {:?}", e);
                SystemError::DbErr(e)
            })?
            .map(|click| {
                if with_fake {
                    click.count_today + click.fake_count
                } else {
                    click.count_today
                }
            });

        return Ok(r.unwrap_or(0));
    }
    //查找某一记录是否存在
    pub async fn exists(
        app_data: &web::Data<AppState>,
        level_id: i32,
        date: NaiveDate,
    ) -> Result<bool, SystemError> {
        let db = app_data.db_conn();

        let r = crate::entity::click::Entity::find()
            .filter(crate::entity::click::Column::IdMood.eq(level_id))
            .filter(crate::entity::click::Column::Date.eq(date))
            .one(db)
            .await
            .map_err(|e| {
                log::error!("Failed to get click count: {:?}", e);
                SystemError::DbErr(e)
            })?
            .map(|click| click.count_today);

        return Ok(r.is_some());
    }
    //增加点击次数
    pub async fn increment_by(
        app_data: &web::Data<AppState>,
        level_id: i32,
        date: NaiveDate,
        incr: i32,
        return_fake: bool,
    ) -> Result<i32, SystemError> {
        let db = app_data.db_conn();
        let sql = r#"
            UPDATE click SET count_today = count_today + ?
            WHERE level_id = ? AND date = ?
        "#;
        let date_str = date.to_string();
        let stmt = sea_orm::Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            sql,
            [
                sea_orm::Value::Int(Some(incr)),
                sea_orm::Value::Int(Some(level_id)),
                sea_orm::Value::String(Some(Box::new(date_str))),
            ],
        );
        let r = db.execute(stmt).await.map_err(|e| {
            log::error!("Failed to increment click count: {:?}", e);
            SystemError::DbErr(e)
        })?;

        if r.rows_affected() == 0 {
            warn!("No click record found,maybe a bug");
            return Err(SystemError::DbErr(DbErr::RecordNotFound(
                "No click record found".to_string(),
            )));
        }

        let new_count = Self::get_click_count(app_data, level_id, date, return_fake).await?;

        Ok(new_count)
    }
    //增加假的点击次数
    pub async fn increment_fake_by(
        app_data: &web::Data<AppState>,
        level_id: i32,
        date: NaiveDate,
        incr: i32,
        return_fake: bool,
    ) -> Result<i32, SystemError> {
        let db = app_data.db_conn();
        let sql = r#"
            UPDATE click SET fake_count = fake_count + ?
            WHERE level_id = ? AND date = ?
        "#;
        let date_str = date.to_string();
        let stmt = sea_orm::Statement::from_sql_and_values(
            DatabaseBackend::MySql,
            sql,
            [
                sea_orm::Value::Int(Some(incr)),
                sea_orm::Value::Int(Some(level_id)),
                sea_orm::Value::String(Some(Box::new(date_str))),
            ],
        );
        let r = db.execute(stmt).await.map_err(|e| {
            log::error!("Failed to increment click count: {:?}", e);
            SystemError::DbErr(e)
        })?;

        if r.rows_affected() == 0 {
            warn!("No click record found,maybe a bug");
            return Err(SystemError::DbErr(DbErr::RecordNotFound(
                "No click record found".to_string(),
            )));
        }

        let new_count = Self::get_click_count(app_data, level_id, date, return_fake).await?;

        Ok(new_count)
    }
}