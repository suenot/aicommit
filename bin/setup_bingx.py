#!/usr/bin/env python3
"""
Скрипт быстрой настройки BingX Position Manager
"""

import json
import os
import sys
from pathlib import Path

def create_config():
    """Создание конфигурационного файла"""
    print("=== Настройка BingX Position Manager ===\n")
    
    config = {
        "api_key": "",
        "secret_key": "",
        "testnet": True,
        "check_interval": 60,
        "profit_threshold": 0.01,
        "partial_close_percent": 0.5,
        "stop_loss_offset": 0.005,
        "min_position_size": 10,
        "enabled_symbols": [],
        "disabled_symbols": [],
        "max_retries": 3,
        "retry_delay": 5,
        "emergency_stop": False,
        "dry_run": True,
        "notifications": {
            "enabled": False,
            "webhook_url": "",
            "telegram_bot_token": "",
            "telegram_chat_id": ""
        },
        "risk_management": {
            "max_daily_trades": 50,
            "max_position_value": 1000,
            "blacklist_on_errors": True,
            "error_threshold": 5
        }
    }
    
    # Запрашиваем основные настройки
    print("1. API настройки:")
    api_key = input("Введите API ключ BingX: ").strip()
    if api_key:
        config["api_key"] = api_key
    
    secret_key = input("Введите секретный ключ BingX: ").strip()
    if secret_key:
        config["secret_key"] = secret_key
    
    print("\n2. Основные настройки:")
    
    testnet = input("Использовать testnet? (y/n) [y]: ").strip().lower()
    config["testnet"] = testnet != 'n'
    
    try:
        interval = input("Интервал проверки в секундах [60]: ").strip()
        if interval:
            config["check_interval"] = int(interval)
    except ValueError:
        pass
    
    try:
        profit = input("Порог прибыли в процентах [1]: ").strip()
        if profit:
            config["profit_threshold"] = float(profit) / 100
    except ValueError:
        pass
    
    try:
        close_percent = input("Процент позиции для закрытия [50]: ").strip()
        if close_percent:
            config["partial_close_percent"] = float(close_percent) / 100
    except ValueError:
        pass
    
    dry_run = input("Включить тестовый режим (без реальных сделок)? (y/n) [y]: ").strip().lower()
    config["dry_run"] = dry_run != 'n'
    
    print("\n3. Уведомления (опционально):")
    enable_notifications = input("Включить уведомления? (y/n) [n]: ").strip().lower()
    if enable_notifications == 'y':
        config["notifications"]["enabled"] = True
        
        webhook = input("Webhook URL (например, Slack): ").strip()
        if webhook:
            config["notifications"]["webhook_url"] = webhook
            
        bot_token = input("Telegram Bot Token: ").strip()
        if bot_token:
            config["notifications"]["telegram_bot_token"] = bot_token
            
        chat_id = input("Telegram Chat ID: ").strip()
        if chat_id:
            config["notifications"]["telegram_chat_id"] = chat_id
    
    # Сохраняем конфигурацию
    config_file = "bingx_config.json"
    try:
        with open(config_file, 'w', encoding='utf-8') as f:
            json.dump(config, f, indent=2, ensure_ascii=False)
        print(f"\n✅ Конфигурация сохранена в {config_file}")
        return True
    except Exception as e:
        print(f"\n❌ Ошибка сохранения конфигурации: {e}")
        return False

def check_dependencies():
    """Проверка зависимостей"""
    print("\n=== Проверка зависимостей ===")
    
    try:
        import requests
        print("✅ requests - установлен")
    except ImportError:
        print("❌ requests - не установлен")
        print("Установите: pip install requests")
        return False
    
    try:
        from bingx.api import BingxAPI
        print("✅ bingx-py - установлен")
    except ImportError:
        print("⚠️  bingx-py - не установлен (будет использоваться прямой API)")
    
    return True

def main():
    print("BingX Position Manager - Быстрая настройка")
    print("=" * 50)
    
    # Проверяем зависимости
    if not check_dependencies():
        print("\nУстановите зависимости и запустите скрипт снова")
        return
    
    # Создаем конфигурацию
    if create_config():
        print("\n=== Настройка завершена ===")
        print("\nСледующие шаги:")
        print("1. Проверьте настройки в bingx_config.json")
        print("2. Запустите тестирование:")
        print("   python bin/bingx_monitor.py start")
        print("3. Проверьте статус:")
        print("   python bin/bingx_monitor.py status")
        print("4. Когда будете готовы к реальной торговле,")
        print("   установите в конфигурации:")
        print("   - \"testnet\": false")
        print("   - \"dry_run\": false")
        print("\n⚠️  ВАЖНО: Сначала протестируйте на testnet!")
    else:
        print("\n❌ Настройка не завершена")

if __name__ == "__main__":
    main()
