# Panther

Беговой GPS трекер с превосходными анимациями!

## Сборка
Ниже представлены скрипты для сборки + запуска приложения на Android. Убедитесь, что вы подключили свое устройство и 
включили режим отладки по USB.
### Сборка Gradle с использованием прекомпилированной rust библиотеки
```bash
./run_gradle_precompiled.sh
```

### Полная сборка с использованием Rust + Gradle

Необходима установка rust toolchain на ваш компьютер в соответствии с https://rustup.rs/

После установки тулчейна сборки, необходимо установить несколько дополнительных инструментов:
```bash
cargo install cargo-ndk
rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android
```

Утилита cargo ndk использует стандартный путь установки или переменные среды ANDROID_NDK_HOME, ANDROID_HOME (Можно поменять в скрипте run_gradle.sh).
Предпологается использование Linux со стандартной установкой android studio. Также необходима установка NDK: Settings > Languages and frameworks >
 Android SDK > SDK Tools > установить галочку на NDK (side by side) > OK.

Запуск полной сборки Rust + Gradle:
```bash
./run_gradle.sh
```
