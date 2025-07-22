# BingX Position Manager - Быстрый старт

## 🚀 Установка за 3 шага

### 1. Установите зависимости
```bash
bash bin/install_bingx_deps.sh
```

### 2. Настройте конфигурацию
```bash
python bin/setup_bingx.py
```

### 3. Запустите скрипт
```bash
python bin/bingx_monitor.py start
```

## 📊 Проверка статуса
```bash
python bin/bingx_monitor.py status
```

## ⚠️ Важные моменты

1. **Сначала тестируйте на testnet!**
   - При настройке выберите "testnet: true"
   - Включите "dry_run: true" для тестирования без сделок

2. **Получите API ключи:**
   - Зайдите на [BingX](https://bingx.com)
   - API Management → Create API Key
   - Включите права на фьючерсную торговлю

3. **Безопасность:**
   - Используйте только необходимые права API
   - Не делитесь ключами
   - Регулярно проверяйте логи

## 🎯 Что делает скрипт

- ✅ Проверяет позиции каждую минуту
- ✅ Добавляет stop loss если отсутствует  
- ✅ При 1% прибыли закрывает 50% позиции
- ✅ Переводит stop loss в безубыток
- ✅ Ведет подробные логи

## 🛠️ Управление

```bash
# Запуск
python bin/bingx_monitor.py start

# Остановка  
python bin/bingx_monitor.py stop

# Аварийная остановка
python bin/bingx_monitor.py emergency

# Статус
python bin/bingx_monitor.py status
```

## 📝 Логи

Все операции записываются в `bingx_position_manager.log`

## 🆘 Помощь

Полная документация: `bin/README_bingx.md`

---
**Создано для проекта aicommit**
