use std::sync::Arc;

use sqlx::Row;
use uuid::Uuid;

use crate::{config::Config, db::DbPool, error::AppError};

#[derive(Clone)]
pub struct PrivacyAccountingService {
    pub db: Arc<DbPool>,
    pub config: Arc<Config>,
}

pub struct BudgetWindow {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

impl PrivacyAccountingService {
    pub fn new(db: Arc<DbPool>, config: Arc<Config>) -> Self {
        Self { db, config }
    }

    pub fn current_window(&self) -> BudgetWindow {
        let now = chrono::Utc::now();
        let start = now - chrono::Duration::hours(self.config.privacy_window_hours);
        BudgetWindow { start, end: now }
    }

    pub async fn ensure_budget_row(
        &self,
        principal_type: &str,
        principal_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let window = self.current_window();
        let id = Uuid::new_v4();
        let id_bytes = id.as_bytes();
        let principal_id_bytes = principal_id.map(|u| u.as_bytes().to_vec());

        let mut conn = self.db.acquire().await.map_err(AppError::DatabaseError)?;
        // Try insert; ignore error on conflict
        let _ = sqlx::query(
            "INSERT INTO privacy_budgets (id, principal_type, principal_id, window_start, window_end, epsilon_total, delta_total, epsilon_spent, delta_spent)
             VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0)"
        )
        .bind(&id_bytes[..])
        .bind(principal_type)
        .bind(principal_id_bytes.as_deref())
        .bind(window.start)
        .bind(window.end)
        .bind(self.config.privacy_epsilon)
        .bind(self.config.privacy_delta)
        .execute(&mut *conn)
        .await;

        Ok(())
    }

    pub async fn spend(
        &self,
        principal_type: &str,
        principal_id: Option<Uuid>,
        endpoint: &str,
        mechanism: &str,
        epsilon_cost: f64,
        delta_cost: f64,
    ) -> Result<bool, AppError> {
        let window = self.current_window();
        let principal_id_bytes = principal_id.map(|u| u.as_bytes().to_vec());

        let mut conn = self.db.acquire().await.map_err(AppError::DatabaseError)?;

        // Fetch current budget within window
        let row_opt = sqlx::query(
            "SELECT id, epsilon_total, delta_total, epsilon_spent, delta_spent
             FROM privacy_budgets
             WHERE principal_type = ? AND (principal_id IS ? OR principal_id = ?) AND window_start <= ? AND window_end >= ?
             LIMIT 1"
        )
        .bind(principal_type)
        .bind(principal_id_bytes.as_deref())
        .bind(principal_id_bytes.as_deref())
        .bind(window.end)
        .bind(window.start)
        .fetch_optional(&mut *conn)
        .await
        .map_err(AppError::DatabaseError)?;

        if let Some(row) = row_opt {
            let id: Vec<u8> = row.get("id");
            let epsilon_total: f64 = row.get("epsilon_total");
            let delta_total: f64 = row.get("delta_total");
            let epsilon_spent: f64 = row.get("epsilon_spent");
            let delta_spent: f64 = row.get("delta_spent");

            let e_rem = epsilon_total - epsilon_spent;
            let d_rem = delta_total - delta_spent;

            if epsilon_cost <= e_rem + f64::EPSILON && delta_cost <= d_rem + f64::EPSILON {
                // Update budget
                sqlx::query(
                    "UPDATE privacy_budgets SET epsilon_spent = epsilon_spent + ?, delta_spent = delta_spent + ? WHERE id = ?"
                )
                .bind(epsilon_cost.max(0.0))
                .bind(delta_cost.max(0.0))
                .bind(&id)
                .execute(&mut *conn)
                .await
                .map_err(AppError::DatabaseError)?;

                // Log spend
                let spend_id = Uuid::new_v4();
                let spend_bytes = spend_id.as_bytes();
                sqlx::query(
                    "INSERT INTO privacy_spend_log (id, principal_type, principal_id, endpoint, mechanism, epsilon_spent, delta_spent)
                     VALUES (?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(&spend_bytes[..])
                .bind(principal_type)
                .bind(principal_id_bytes.as_deref())
                .bind(endpoint)
                .bind(mechanism)
                .bind(epsilon_cost)
                .bind(delta_cost)
                .execute(&mut *conn)
                .await
                .map_err(AppError::DatabaseError)?;

                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            // No budget in window yet; create and retry once
            self.ensure_budget_row(principal_type, principal_id).await?;
            Ok(false)
        }
    }
}


