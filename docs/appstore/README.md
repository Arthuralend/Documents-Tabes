# FS Office Write в Mac App Store

Пошаговый путь от репозитория до публикации Write в Mac App Store. Отмечено, что можно сделать
только вам (аккаунт Apple, оплата, сертификаты), что уже готово в репозитории и что ещё нужно
доработать в коде.

## Что уже готово в репозитории

| Файл | Зачем |
|---|---|
| [`packaging/macos/appstore/build.sh`](../../packaging/macos/appstore/build.sh) | Собирает универсальный (Apple Silicon + Intel) `Write.app` в песочнице, подписывает его сертификатом Apple Distribution с профилем Mac App Store, упаковывает в `.pkg` с подписью Mac Installer Distribution и (с `--upload`) отправляет в App Store Connect |
| [`packaging/macos/appstore/entitlements.plist`](../../packaging/macos/appstore/entitlements.plist) | Права приложения в песочнице: выбранные пользователем файлы, закладки, микрофон для диктовки, печать |
| [`metadata-ru.md`](metadata-ru.md), [`metadata-en.md`](metadata-en.md) | Тексты карточки: название, подзаголовок, описание, ключевые слова, что нового |
| [`privacy-policy.md`](privacy-policy.md) | Политика конфиденциальности (нужна ссылка на неё в App Store Connect) |
| `assets/app-icon/wordcraft-1024.png` | Значок 1024×1024 (в `.icns` внутри приложения — все размеры) |

## 1. Аккаунт разработчика (только вы)

1. Вступите в **Apple Developer Program**: <https://developer.apple.com/programs/> — 99 $ в год.
   Для продажи под названием компании (FantasticSoftware) нужна регистрация как организации
   (номер D-U-N-S); как частное лицо — продавцом будет указано ваше имя.
2. В App Store Connect подпишите соглашения (Agreements, Tax, and Banking). Для бесплатного
   приложения хватит Free Apps Agreement; для платного — заполните налоги и банк.

## 2. Идентификатор, сертификаты, профиль (только вы, один раз)

На <https://developer.apple.com/account/resources/>:

1. **Identifiers → App IDs → +**: тип *App*, платформа macOS, Bundle ID **explicit**, например
   `com.fantasticsoftware.write` (его же передайте сборке в `APPSTORE_BUNDLE_ID`). Отдельных
   возможностей (Capabilities) включать не нужно.
2. **Certificates → +**:
   - *Apple Distribution* — подпись приложения;
   - *Mac Installer Distribution* — подпись `.pkg`.
   Создайте запросы (CSR) в «Связке ключей» на Mac, скачайте сертификаты и откройте их, чтобы они
   легли в связку ключей. Проверить: `security find-identity -v -p codesigning`.
3. **Profiles → +** → *Mac App Store Connect* (Distribution) → ваш App ID → сертификат Apple
   Distribution. Скачайте `.provisionprofile`.

## 3. Запись в App Store Connect (только вы)

<https://appstoreconnect.apple.com> → **Мои приложения → + → Новое приложение**:

- Платформа: macOS. Bundle ID — тот, что создан выше. SKU — любой, например `fs-office-write`.
- **Название.** В App Store имя должно быть уникальным, и «Write» почти наверняка занято.
  Предлагаем **«FS Office Write»** (это и есть полное название), подзаголовок — из
  [`metadata-ru.md`](metadata-ru.md). Название на значке и в Dock остаётся «Write».
- Основной язык: русский; добавьте английскую локализацию из [`metadata-en.md`](metadata-en.md).
- Категория: **Производительность** (Productivity), дополнительная — **Бизнес**.
- **URL политики конфиденциальности** — обязателен. Опубликуйте
  [`privacy-policy.md`](privacy-policy.md) (например, GitHub Pages этого репозитория или любая
  страница) и вставьте ссылку.
- **URL поддержки** — например, <https://github.com/Arthuralend/Documents-Tabes/issues> или
  страница с адресом armenoid@gmail.com.
- **Конфиденциальность приложения (App Privacy):** «Данные не собираются» — Write не отправляет
  никаких данных: всё хранится на компьютере пользователя, диктовка распознаётся средствами macOS.
- **Возрастной рейтинг:** анкета — везде «Нет» → 4+.
- **Экспортное соответствие (шифрование):** Write пользуется только стандартной криптографией (для
  открытия документов Word, защищённых паролем) — это освобождённое шифрование. В `Info.plist`
  сборка ставит `ITSAppUsesNonExemptEncryption = NO`; если App Store Connect спросит, отвечайте,
  что приложение использует только освобождённое шифрование.
- **Цена:** бесплатно или по вашему выбору.

### Снимки экрана

Нужен хотя бы один снимок с соотношением 16:10, допустимые размеры: **1280×800, 1440×900,
2560×1600 или 2880×1800** (PNG/JPEG, без прозрачности), до 10 штук. Советуем 3–5 снимков на
2880×1800: документ на главной вкладке, диктовка со «шлейфом» новых слов, проверка орфографии на
русском и английском, два окна рядом, сохранение в PDF.

## 4. Сборка и отправка (на Mac)

```bash
export APPSTORE_BUNDLE_ID=com.fantasticsoftware.write
export APPSTORE_APP_IDENTITY="Apple Distribution: <Имя> (<TEAMID>)"
export APPSTORE_INSTALLER_IDENTITY="3rd Party Mac Developer Installer: <Имя> (<TEAMID>)"
export APPSTORE_PROFILE=~/Downloads/FS_Office_Write_Mac_App_Store.provisionprofile
packaging/macos/appstore/build.sh
```

Готовый `dist/release/FS-Office-Write-<версия>-appstore.pkg` отправьте приложением
**Transporter** (бесплатно в Mac App Store) или сразу из скрипта с ключом App Store Connect API
(`APPSTORE_API_KEY_ID`, `APPSTORE_API_ISSUER`, файл `.p8` в `~/.appstoreconnect/private_keys/`) и
флагом `--upload`. Через 10–30 минут сборка появится в App Store Connect → TestFlight; выберите её
в версии и нажмите **Отправить на проверку**.

## 5. Что нужно доработать в коде до отправки

В App Store приложение работает в **песочнице**: оно видит только те файлы, которые пользователь
выбрал сам. Обычная версия (DMG с GitHub) песочницы не имеет, поэтому эти места пока рассчитаны на
полный доступ к диску:

1. **Второе окно — отдельный процесс.** Новый документ в новом окне сейчас запускает ещё одну копию
   программы и передаёт ей путь к файлу. В песочнице разрешение на файл не передаётся новому
   процессу, и он не сможет открыть файл. Нужно открывать окна в одном процессе или передавать
   закладку безопасности (security-scoped bookmark).
2. **Недавние документы** хранятся как пути. После перезапуска в песочнице они не откроются —
   нужно хранить закладки безопасности (право `files.bookmarks.app-scope` уже включено).
3. **Печать** идёт через системную команду печати; в песочнице её надо проверить
   (право `print` включено).
4. **Канал управления** (`--control`, MCP) открывает локальный сетевой порт; в версии для App
   Store его лучше отключить (сетевые права не запрошены).
5. **Table** читает настройки Write, чтобы взять ту же цветовую тему. В песочнице у каждой
   программы своя папка, и Table их не увидит — для Table в App Store нужна общая группа
   приложений (App Group) или своя тема.

Проверка ревьюеров Apple обычно занимает 1–3 дня. Частые причины отказа: падение при запуске,
нерабочие ссылки на политику и поддержку, кнопки, которые ничего не делают, и названия других
компаний в описании.
