// The file storage works only on Linux, MacOS, and Windows
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
compile_error!("Unsupported operating system");

pub mod file;
pub mod layout;
pub mod path;

pub mod fs;
pub mod types;
pub mod utils;

pub mod mount;
pub mod storage;

/*
    path == `media/Fm/sd/FmsdPLuY1A/original`

    // --- Сохранение файла ---
    let staged = storage.stage_file(path).await?;   // Создание staged файла

    let writer = staged.writer().await?;            // Создание врайтера для staged файла
    writer.write_chunked_callback(                  // Запись стрима во врайтер
        stream,
        |chunk| { ... } // Для каждого чанка вызывается каллбэк
    ).await?;

    writer.flush().await?;                          // Завершение записи во врайтер
    stage.commit().await?;                          // Коммит staged файла в хранилище (`FmsdPLuY1A/~original-GYre0rpM` -> `FmsdPLuY1A/original`)

    // --- Сохранение файла (В одну строку через StorageExt) ---
    let staged = storage.upload(path, stream).await?;   // Создание staged файла и запись в него стрима
    staged.commit().await?;                             // Коммит staged файла в хранилище (`FmsdPLuY1A/~original-GYre0rpM` -> `FmsdPLuY1A/original`)

    // --- Загрузка файла через внешний писатель ---
    let staged = storage.stage_file(path).await?;   // Создание staged файла
    let uploader = staged.foreign_uploader();       // Превращение staged в foreign загрузчик (Позволяет через API увидеть реальный путь)

    ffmpeg::transcode_video( // Транскодирование видео в foreign_uploader через ffmpeg
        input,
        uploader.path()
    )?;

    uploader.finalize().await?;                     // Финализирует uploader. Проверяет, было ли что-то загружено
    let res = staged.commit().await?;               // Коммит staged файла в хранилище (`FmsdPLuY1A/~preview-GYre0rpM` -> `FmsdPLuY1A/preview`)

    // --- Загрузка директории через внешний писатель ---
    let staged = storage.stage_dir(path).await?;    // Создание staged директории
    let uploader = staged.foreign_uploader();       // Превращение staged в foreign загрузчик (Позволяет через API увидеть реальный путь)

    ffmpeg::build_hls(
        input,
        uploader.path(),
        "index.m3u8"
    )?;

    uploader.finalize().await?;                     // Финализирует uploader
    let res = uploader.commit().await?              // Коммит staged директории в хранилище (`FmsdPLuY1A/hls/~1080-GYre0rpM/` -> `FmsdPLuY1A/hls/1080`)

    // --- Загрузка файла через внешний писатель (StorageExt) ---
    let uploader = storage.foreign_uploader(path).await?; // Создание внешнего загрузчика

    < ... >

    let res = uploader.commit().await?;             // Коммит staged файла в хранилище (`FmsdPLuY1A/~preview-GYre0rpM` -> `FmsdPLuY1A/preview`)

    // --- Получение файла локально ---
    let local = storage.local_file(path).await?;    // Получает файл локально

    let path = local.path(); // Получение локального пути файла
    let size = local.size(); // Получение размера файла

    // --- Чтение файла ---
    let reader = storage.reader(path).await?;                            // Открытие ридера
    let reader_seek = storage.reader_seek(path, 90, Some(500)).await?;   // Открытие ридера 90-500 байты

    let meta = storage.meta(path).await?;                                // Возвращает метаданные файла

    // --- Проверка файла ---
    storage.exists(path).await?;                                // Возвращает `true`, если файл существует
    storage.checksum(path, ChecksumAlgorithm::Sha256).await?;   // Возвращает checksum для файла хранилища

    // --- Перемещение файла ---
    storage.rename(path, dest).await?;                           // Изменение ключа файла в хранилище

    // --- Чтение HLS стрима (StorageHlsExt; использует базовые операции чтения и парсер Hls manifest) ---
    let reader = storage.hls_reader(path).await?;   // Требует ключ к hls-manifest

    let manifest = reader.manifest().await?;        // Читает манифест hls

    for seg in reader.segments_iter().await? {      // Читает сегменты через итератор
        < ... >
    }

    let segment = reader.segment(500).await?;       // Пробует получить ридер на `пятисотый` сегмент (`<path>/500.ts`)

    // --- Информация о хранилище ---
    storage.is_local();                             // Возвращает `true`, если хранилище использует локальный диск

    let stats = storage.disk_usage().await?;        // Возвращает статистику использования диска
*/
