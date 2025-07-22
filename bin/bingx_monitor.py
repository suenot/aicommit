#!/usr/bin/env python3
"""
BingX Position Manager Monitor
Скрипт для мониторинга и управления основным скриптом
"""

import json
import os
import sys
import time
import subprocess
from datetime import datetime, timedelta
from pathlib import Path

CONFIG_FILE = "bingx_config.json"
LOG_FILE = "bingx_position_manager.log"
PID_FILE = "bingx_manager.pid"

class BingXMonitor:
    def __init__(self):
        self.config_path = CONFIG_FILE
        
    def load_config(self):
        """Загрузка конфигурации"""
        if not os.path.exists(self.config_path):
            print(f"Файл конфигурации {self.config_path} не найден")
            return None
        
        try:
            with open(self.config_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception as e:
            print(f"Ошибка загрузки конфигурации: {e}")
            return None
            
    def save_config(self, config):
        """Сохранение конфигурации"""
        try:
            with open(self.config_path, 'w', encoding='utf-8') as f:
                json.dump(config, f, indent=2, ensure_ascii=False)
            return True
        except Exception as e:
            print(f"Ошибка сохранения конфигурации: {e}")
            return False
            
    def is_running(self):
        """Проверка, запущен ли основной скрипт"""
        if not os.path.exists(PID_FILE):
            return False
            
        try:
            with open(PID_FILE, 'r') as f:
                pid = int(f.read().strip())
                
            # Проверяем, существует ли процесс
            try:
                os.kill(pid, 0)
                return True
            except OSError:
                # Процесс не существует, удаляем PID файл
                os.remove(PID_FILE)
                return False
        except:
            return False
            
    def start_manager(self):
        """Запуск основного скрипта"""
        if self.is_running():
            print("Скрипт уже запущен")
            return False
            
        try:
            # Запускаем скрипт в фоне
            process = subprocess.Popen([
                sys.executable, 
                "bin/bingx_position_manager.py"
            ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            
            # Сохраняем PID
            with open(PID_FILE, 'w') as f:
                f.write(str(process.pid))
                
            print(f"Скрипт запущен с PID: {process.pid}")
            return True
        except Exception as e:
            print(f"Ошибка запуска: {e}")
            return False
            
    def stop_manager(self):
        """Остановка основного скрипта"""
        if not self.is_running():
            print("Скрипт не запущен")
            return False
            
        try:
            with open(PID_FILE, 'r') as f:
                pid = int(f.read().strip())
                
            os.kill(pid, 15)  # SIGTERM
            time.sleep(2)
            
            # Проверяем, завершился ли процесс
            try:
                os.kill(pid, 0)
                # Если процесс все еще существует, принудительно завершаем
                os.kill(pid, 9)  # SIGKILL
            except OSError:
                pass
                
            os.remove(PID_FILE)
            print("Скрипт остановлен")
            return True
        except Exception as e:
            print(f"Ошибка остановки: {e}")
            return False
            
    def emergency_stop(self):
        """Аварийная остановка"""
        config = self.load_config()
        if config:
            config['emergency_stop'] = True
            if self.save_config(config):
                print("Активирована аварийная остановка")
                return True
        return False
        
    def resume(self):
        """Снятие аварийной остановки"""
        config = self.load_config()
        if config:
            config['emergency_stop'] = False
            if self.save_config(config):
                print("Аварийная остановка отключена")
                return True
        return False
        
    def status(self):
        """Показать статус"""
        print("=== BingX Position Manager Status ===")
        
        # Статус процесса
        if self.is_running():
            print("✅ Статус: Запущен")
        else:
            print("❌ Статус: Остановлен")
            
        # Конфигурация
        config = self.load_config()
        if config:
            print(f"🔧 Testnet: {'Да' if config.get('testnet', True) else 'Нет'}")
            print(f"⏱️  Интервал: {config.get('check_interval', 60)} сек")
            print(f"📈 Порог прибыли: {config.get('profit_threshold', 0.01):.1%}")
            print(f"🛑 Аварийная остановка: {'Да' if config.get('emergency_stop', False) else 'Нет'}")
            print(f"🧪 Тестовый режим: {'Да' if config.get('dry_run', False) else 'Нет'}")
            
        # Логи
        if os.path.exists(LOG_FILE):
            stat = os.stat(LOG_FILE)
            size = stat.st_size / 1024  # KB
            modified = datetime.fromtimestamp(stat.st_mtime)
            print(f"📝 Лог файл: {size:.1f} KB, изменен {modified.strftime('%Y-%m-%d %H:%M:%S')}")
            
            # Последние строки лога
            try:
                with open(LOG_FILE, 'r', encoding='utf-8') as f:
                    lines = f.readlines()
                    if lines:
                        print("📋 Последние записи:")
                        for line in lines[-3:]:
                            print(f"   {line.strip()}")
            except:
                pass
        else:
            print("📝 Лог файл не найден")
            
    def show_help(self):
        """Показать справку"""
        print("""
BingX Position Manager Monitor

Команды:
  start     - Запустить менеджер позиций
  stop      - Остановить менеджер позиций
  restart   - Перезапустить менеджер позиций
  status    - Показать статус
  emergency - Активировать аварийную остановку
  resume    - Снять аварийную остановку
  help      - Показать эту справку

Примеры:
  python bin/bingx_monitor.py start
  python bin/bingx_monitor.py status
  python bin/bingx_monitor.py emergency
        """)

def main():
    monitor = BingXMonitor()
    
    if len(sys.argv) < 2:
        monitor.show_help()
        return
        
    command = sys.argv[1].lower()
    
    if command == "start":
        monitor.start_manager()
    elif command == "stop":
        monitor.stop_manager()
    elif command == "restart":
        monitor.stop_manager()
        time.sleep(1)
        monitor.start_manager()
    elif command == "status":
        monitor.status()
    elif command == "emergency":
        monitor.emergency_stop()
    elif command == "resume":
        monitor.resume()
    elif command == "help":
        monitor.show_help()
    else:
        print(f"Неизвестная команда: {command}")
        monitor.show_help()

if __name__ == "__main__":
    main()
