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

## 4. Сборка и отправка — на GitHub, без своего Mac

Сборку для App Store делает GitHub (workflow [`.github/workflows/appstore.yml`](../../.github/workflows/appstore.yml)):
он собирает только Write, подписывает его, упаковывает в `.pkg` и отправляет в App Store Connect.

1. **Экспорт сертификатов.** В «Связке ключей» на Mac найдите «Apple Distribution: …» → правый
   клик → «Экспортировать…» → `.p12`, задайте пароль. Так же экспортируйте
   «3rd Party Mac Developer Installer: …» (или «Mac Installer Distribution: …») с тем же паролем.
2. **Ключ App Store Connect API.** App Store Connect → «Пользователи и доступ» → «Интеграции» →
   «App Store Connect API» → «+», роль **App Manager**. Скачайте `AuthKey_XXXX.p8` (скачать можно
   только один раз), запишите **Key ID** и **Issuer ID**.
3. **Секреты GitHub**: <https://github.com/Arthuralend/Documents-Tabes/settings/secrets/actions> →
   «New repository secret». Base64 файла: в Терминале `base64 -i файл | pbcopy`, затем вставить.

   | Секрет | Значение |
   |---|---|
   | `APPSTORE_APP_CERT_P12` | base64 `.p12` Apple Distribution |
   | `APPSTORE_INSTALLER_CERT_P12` | base64 `.p12` Mac Installer Distribution |
   | `APPSTORE_CERT_PASSWORD` | пароль от обоих `.p12` |
   | `APPSTORE_PROFILE` | base64 файла `.provisionprofile` |
   | `APPSTORE_API_KEY_ID` | Key ID |
   | `APPSTORE_API_ISSUER` | Issuer ID |
   | `APPSTORE_API_KEY_P8` | текст файла `AuthKey_XXXX.p8` целиком |

4. **Запуск**: GitHub → Actions → «App Store (Write for Mac)» → «Run workflow» (Bundle ID —
   `com.fantasticsoftware.write`, если вы зарегистрировали его). Через 30–40 минут сборка
   появится в App Store Connect (раздел TestFlight), ещё через 10–30 минут — пройдёт обработку.
   Готовый `.pkg` также лежит в результатах запуска (Artifacts) — его можно отправить вручную
   программой **Transporter**.
5. В App Store Connect откройте версию 1.0 → «Сборка» → «+» → выберите сборку → **Отправить на
   проверку**.

Собрать на своём Mac тоже можно: `packaging/macos/appstore/build.sh` (переменные — в начале
скрипта).

## 5. Песочница: что уже сделано и что проверить

В App Store Write работает в **песочнице**: он видит только те файлы, которые пользователь выбрал сам.

- **Сделано:** документы, которые открыли или сохранили, запоминаются через закладки безопасности
  (`crates/sandbox`, `apps/wordcraft/src/bookmarks.rs`). Поэтому недавние документы открываются
  после перезапуска, а документ, открытый в новом окне (это отдельный процесс), там доступен.
- **Сделано:** в песочнице нельзя создать временный файл рядом с документом. Поэтому сохранение
  пишет документ на месте, а при ошибке возвращает прежнюю версию (`crates/engine/src/io/safe_save.rs`).
- **Проверить в TestFlight перед отправкой:** открыть, сохранить, «Сохранить как», недавние документы
  после перезапуска, второе окно, печать, диктовку, экспорт PDF.

Проверка ревьюеров Apple обычно занимает 1–3 дня. Частые причины отказа: падение при запуске,
нерабочие ссылки на политику и поддержку, кнопки, которые ничего не делают, и названия других
компаний в описании.
