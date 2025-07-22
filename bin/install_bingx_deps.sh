#!/bin/bash

echo "Установка зависимостей для BingX Position Manager..."

# Проверяем наличие pip
if ! command -v pip &> /dev/null; then
    echo "Ошибка: pip не найден. Установите Python и pip."
    exit 1
fi

# Устанавливаем основные зависимости
echo "Установка requests..."
pip install requests

# Пытаемся установить bingx-py
echo "Попытка установки bingx-py..."
pip install bingx-py || echo "Предупреждение: bingx-py не установлен, будет использоваться прямой API"

echo "Установка завершена!"
echo ""
echo "Теперь запустите скрипт:"
echo "python bin/bingx_position_manager.py"
