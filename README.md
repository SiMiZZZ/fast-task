# Fast Task 🚀

**Fast Task** - это быстрый и удобный CLI инструмент для создания задач в Jira, написанный на Rust. Забудьте о медленном веб-интерфейсе - создавайте задачи прямо из терминала!

## ✨ Возможности

- 🔧 **Интерактивная настройка** - пошаговая конфигурация подключения к Jira
- 📋 **Управление проектами** - добавление и просмотр ваших проектов
- 🎯 **Создание задач** - интерактивное создание задач с выбором реальных типов из API
- 🔍 **Проверка соединения** - тестирование подключения к Jira
- 📌 **Мои задачи** - просмотр всех задач, назначенных на вас, прямо в терминале
- ⚡ **Быстрота** - мгновенное создание задач без ожидания загрузки веб-интерфейса

## 🚀 Установка

### Требования
- Rust 1.70+ (установите с [rustup.rs](https://rustup.rs/))

### Установка из исходного кода

```bash
# Клонируйте репозиторий
git clone https://github.com/SiMiZZZ/fast-task.git
cd fast-task

# Соберите проект
cargo build --release

# Установите в систему
cargo install --path .
```

После установки команда `fast-task` будет доступна глобально в вашей системе.

## ⚙️ Быстрый старт

### 1. Настройка подключения к Jira

```bash
fast-task config
```

Вам потребуется:
- **URL Jira** (например: `https://company.atlassian.net`)
- **Email** - ваш email в Jira
- **API Token** - создайте в [Jira Account Settings → Security → API tokens](https://id.atlassian.com/manage-profile/security/api-tokens)

### 2. Добавьте проект

```bash
fast-task add-project
```

Укажите ключ проекта (например: `PROJ`) и название для отображения.

### 3. Создайте первую задачу

```bash
fast-task create
```

Приложение проведет вас через интерактивный процесс создания задачи.

## 📖 Доступные команды

| Команда | Описание |
|---------|----------|
| `fast-task config` | Настройка подключения к Jira |
| `fast-task add-project` | Добавление нового проекта |
| `fast-task list-projects` | Просмотр настроенных проектов |
| `fast-task test` | Проверка соединения с Jira |
| `fast-task create` | Создание новой задачи |
| `fast-task my-issues` | Просмотр задач, назначенных на вас |

## 💡 Примеры использования

### Проверка конфигурации
```bash
$ fast-task test
🔍 Testing Jira connection...
✅ Connection successful!
   URL: https://company.atlassian.net
   Email: user@company.com
```

### Просмотр проектов
```bash
$ fast-task list-projects
Configured projects:
  WEB - Company Website
  API - Backend API
  MOBILE - Mobile App
```

### Просмотр назначенных задач
```bash
$ fast-task my-issues
🔍 Fetching your assigned issues...
Found 3 issue(s):

WEB-42  | In Progress | Fix responsive layout on mobile | https://company.atlassian.net/browse/WEB-42
WEB-57  | To Do       | Add dark mode support           | https://company.atlassian.net/browse/WEB-57
API-103 | In Review   | Refactor auth middleware        | https://company.atlassian.net/browse/API-103
```

Доступные флаги:
- `--project <KEY>` — фильтр по конкретному проекту
- `--all` — включить завершённые задачи (по умолчанию показываются только активные)

```bash
# Только задачи проекта WEB
$ fast-task my-issues --project WEB

# Все задачи, включая завершённые
$ fast-task my-issues --all

# Комбинация флагов
$ fast-task my-issues --project WEB --all
```

### Создание задачи
```bash
$ fast-task create
🎯 Creating a new Jira issue 

? Which project? WEB - Company Website
✓ Selected project: WEB (Company Website)
? Issue title: Fix responsive layout on mobile
? Add description? No
🔍 Fetching available issue types for project WEB...
✅ Found 4 issue type(s) for project WEB
? Issue type: Bug - A problem which impairs or prevents functionality
? Create this issue? Yes

🚀 Creating issue...
✅ Issue created successfully!
🔗 https://company.atlassian.net/browse/WEB-123
```

## 🛠 Технические детали

- **Язык**: Rust 🦀
- **Async runtime**: Tokio для эффективной работы с сетью
- **HTTP клиент**: Reqwest для взаимодействия с Jira API
- **CLI интерфейс**: Clap для парсинга команд
- **Интерактивность**: Inquire для красивых промптов

## 🤝 Участие в разработке

Приветствуются любые улучшения! Создавайте issues и pull requests.




