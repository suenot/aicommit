
# Фичи, которые я (заказчик) хочу, чтобы были сделаны (этот список не редактируется):
- написать тесты на каждую фичу, чтобы больше на заливал код, который ломает тесты
- aicommit --by-file # заливать коммиты по 1 файлу
- aicommit --by-feature # разделять коммиты по фичам
- добавить проверку на базовые .gitignore файлы (добавлять базовый набор в ~/commit.json, чтобы пользователь мог потом сам его отредактировать)
- предложить добавить в gitignore (а если его нету, то создать)?
? ollama провайдер для локального использования (убедиться, что работает)
- aicommit --watch 1m # проверять раз в 1m и если есть файлы для коммита, то коммитить
- aicommit --watch 1m --wait-for-edit 30s # убеждается, что файл не меняется уже 30 секунд, только после этого заливать
- флага для обновление версий и в nodejs, python и других языках (package.json, requirements.txt, etc)
- aicommit --init
commit --init
- aicommit --generate (а потом спрашивай, нравится или нет: вариант отредактировать в vim, сгенерировать еще раз, добавить дополнительного контекста и сгенерировать еще раз). Если нравится, то коммитить.
- aicommit --push
    - проверить как работает в других ветках, чтобы факин шит не заливал коммиты из одной ветки в master ветку
- auto push
- check pull 
- auto pull ?
- https://www.conventionalcommits.org/en/v1.0.0/ либо свои правила по формирвоанию коммита (может передать на вход fix #<id>, а все остальное додумает llm)
- после всех фич сделать тесты
- придумать иконку
- ридми довести до совершенства
- добавить проект в список openrouter (чтобы получить немного аудитории)