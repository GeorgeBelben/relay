use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!("SELECT value FROM settings WHERE key = ?", key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.value))
}

pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO settings (key, value) VALUES (?, ?)
           ON CONFLICT (key) DO UPDATE SET value = excluded.value"#,
        key,
        value,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// --- Typed general settings (REL-108), ported from the Electron MVP's electron-store schema
// (lib/store.ts) -- the raw get/set above has no notion of defaults or validation, so every
// known setting gets a typed accessor pair here instead of scattering magic key strings and
// ad-hoc fallback logic across call sites (commands::emulator's RetroArch cores path lookup
// used to hard-error when unset; this is what lets it default sensibly instead, matching the
// MVP's own default for a stock apt install).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControllerType {
    Xbox,
    Playstation,
    Switch,
    Generic,
}

impl ControllerType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Xbox => "xbox",
            Self::Playstation => "playstation",
            Self::Switch => "switch",
            Self::Generic => "generic",
        }
    }

    // An unrecognized stored value (a legacy/corrupted setting) falls back to the schema's own
    // default rather than erroring -- same leniency the MVP's electron-store enum validation
    // didn't really have, but matches how every other default below behaves on a missing key.
    fn parse(value: &str) -> Self {
        match value {
            "playstation" => Self::Playstation,
            "switch" => Self::Switch,
            "generic" => Self::Generic,
            _ => Self::Xbox,
        }
    }
}

const DEFAULT_RETROARCH_CORES_PATH: &str = "/usr/lib/x86_64-linux-gnu/libretro";
const DEFAULT_SOUND_VOLUME: i64 = 70;

#[derive(Debug, Clone, Serialize)]
pub struct GeneralSettings {
    pub onboarding_completed: bool,
    pub controller_type: ControllerType,
    pub active_profile_id: Option<String>,
    pub retroarch_cores_path: String,
    pub wallpaper: Option<String>,
    pub sound_volume: i64,
    pub rumble_enabled: bool,
}

pub async fn get_general_settings(pool: &SqlitePool) -> Result<GeneralSettings, sqlx::Error> {
    Ok(GeneralSettings {
        onboarding_completed: get(pool, "onboardingCompleted").await?.as_deref() == Some("true"),
        controller_type: ControllerType::parse(get(pool, "controllerType").await?.as_deref().unwrap_or("xbox")),
        // "" means "no active profile" in the MVP's schema -- modeled as None here instead of a
        // magic empty string, same reasoning as wallpaper below.
        active_profile_id: get(pool, "activeProfileId").await?.filter(|v| !v.is_empty()),
        retroarch_cores_path: get(pool, "retroarchCoresPath").await?.unwrap_or_else(|| DEFAULT_RETROARCH_CORES_PATH.to_string()),
        wallpaper: get(pool, "wallpaper").await?.filter(|v| !v.is_empty()),
        sound_volume: get(pool, "soundVolume")
            .await?
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(DEFAULT_SOUND_VOLUME)
            .clamp(0, 100),
        rumble_enabled: get(pool, "rumbleEnabled").await?.map(|v| v == "true").unwrap_or(true),
    })
}

pub async fn set_onboarding_completed(pool: &SqlitePool, value: bool) -> Result<(), sqlx::Error> {
    set(pool, "onboardingCompleted", if value { "true" } else { "false" }).await
}

pub async fn set_controller_type(pool: &SqlitePool, value: ControllerType) -> Result<(), sqlx::Error> {
    set(pool, "controllerType", value.as_str()).await
}

pub async fn set_active_profile_id(pool: &SqlitePool, value: Option<&str>) -> Result<(), sqlx::Error> {
    set(pool, "activeProfileId", value.unwrap_or("")).await
}

pub async fn set_retroarch_cores_path(pool: &SqlitePool, value: &str) -> Result<(), sqlx::Error> {
    set(pool, "retroarchCoresPath", value).await
}

pub async fn set_wallpaper(pool: &SqlitePool, value: Option<&str>) -> Result<(), sqlx::Error> {
    set(pool, "wallpaper", value.unwrap_or("")).await
}

pub async fn set_sound_volume(pool: &SqlitePool, value: i64) -> Result<(), sqlx::Error> {
    set(pool, "soundVolume", &value.clamp(0, 100).to_string()).await
}

pub async fn set_rumble_enabled(pool: &SqlitePool, value: bool) -> Result<(), sqlx::Error> {
    set(pool, "rumbleEnabled", if value { "true" } else { "false" }).await
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn throwaway_pool() -> (SqlitePool, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let options = sqlx::sqlite::SqliteConnectOptions::new().filename(&db_path).create_if_missing(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new().connect_with(options).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        (pool, dir)
    }

    #[tokio::test]
    async fn general_settings_default_when_nothing_is_set() {
        let (pool, _dir) = throwaway_pool().await;

        let settings = get_general_settings(&pool).await.unwrap();

        assert!(!settings.onboarding_completed);
        assert_eq!(settings.controller_type, ControllerType::Xbox);
        assert_eq!(settings.active_profile_id, None);
        assert_eq!(settings.retroarch_cores_path, DEFAULT_RETROARCH_CORES_PATH);
        assert_eq!(settings.wallpaper, None);
        assert_eq!(settings.sound_volume, DEFAULT_SOUND_VOLUME);
        assert!(settings.rumble_enabled);
    }

    #[tokio::test]
    async fn each_setter_is_reflected_by_get_general_settings() {
        let (pool, _dir) = throwaway_pool().await;

        set_onboarding_completed(&pool, true).await.unwrap();
        set_controller_type(&pool, ControllerType::Playstation).await.unwrap();
        set_active_profile_id(&pool, Some("profile-1")).await.unwrap();
        set_retroarch_cores_path(&pool, "/opt/retroarch/cores").await.unwrap();
        set_wallpaper(&pool, Some("space.jpg")).await.unwrap();
        set_sound_volume(&pool, 42).await.unwrap();
        set_rumble_enabled(&pool, false).await.unwrap();

        let settings = get_general_settings(&pool).await.unwrap();

        assert!(settings.onboarding_completed);
        assert_eq!(settings.controller_type, ControllerType::Playstation);
        assert_eq!(settings.active_profile_id.as_deref(), Some("profile-1"));
        assert_eq!(settings.retroarch_cores_path, "/opt/retroarch/cores");
        assert_eq!(settings.wallpaper.as_deref(), Some("space.jpg"));
        assert_eq!(settings.sound_volume, 42);
        assert!(!settings.rumble_enabled);
    }

    #[tokio::test]
    async fn clearing_active_profile_id_and_wallpaper_round_trips_as_none() {
        let (pool, _dir) = throwaway_pool().await;

        set_active_profile_id(&pool, Some("profile-1")).await.unwrap();
        set_wallpaper(&pool, Some("space.jpg")).await.unwrap();
        set_active_profile_id(&pool, None).await.unwrap();
        set_wallpaper(&pool, None).await.unwrap();

        let settings = get_general_settings(&pool).await.unwrap();
        assert_eq!(settings.active_profile_id, None);
        assert_eq!(settings.wallpaper, None);
    }

    #[tokio::test]
    async fn sound_volume_clamps_to_0_100_on_set() {
        let (pool, _dir) = throwaway_pool().await;

        set_sound_volume(&pool, 500).await.unwrap();
        assert_eq!(get_general_settings(&pool).await.unwrap().sound_volume, 100);

        set_sound_volume(&pool, -5).await.unwrap();
        assert_eq!(get_general_settings(&pool).await.unwrap().sound_volume, 0);
    }

    #[tokio::test]
    async fn an_unrecognized_stored_controller_type_falls_back_to_the_default() {
        let (pool, _dir) = throwaway_pool().await;

        set(&pool, "controllerType", "some-future-controller").await.unwrap();
        assert_eq!(get_general_settings(&pool).await.unwrap().controller_type, ControllerType::Xbox);
    }
}
