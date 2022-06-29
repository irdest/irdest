FROM ubuntu:20.04

LABEL maintainer="Katharina Fey, irdest community"

# Get the latest version from https://developer.android.com/studio/index.html
ENV ANDROID_SDK_TOOLS_VERSION="8512546"
ENV ANDROID_NDK_VERSION="23.2.8568313"
ENV ANDROID_COMPILE_SDK="32"
ENV ANDROID_BUILD_TOOLS="32.0.0"

ENV ANDROID_HOME="/opt/android-sdk" \
    ANDROID_SDK_ROOT="${ANDROID_HOME}" \
    JAVA_HOME=/usr/lib/jvm/java-11-openjdk-amd64/

# Set locale
ENV LANG="en_US.UTF-8" \
    LANGUAGE="en_US.UTF-8" \
    LC_ALL="en_US.UTF-8"

ENV DEBIAN_FRONTEND="noninteractive" \
    TERM=dumb \
    DEBIAN_FRONTEND=noninteractive

ENV PATH="$PATH:${ANDROID_SDK_HOME}/platform-tools/:${ANDROID_HOME}/cmdline-tools/latest/bin/:/root/.cargo/bin/"

WORKDIR /irdest/clients/android-vpn

# Installing packages
RUN apt-get clean && \
    apt-get update -qq && \
    apt-get install -qq apt-utils locales > /dev/null && \
    locale-gen "$LANG" > /dev/null && \
    apt-get install -qq --no-install-recommends \
        locales\
        build-essential \
        cmake \
        curl \
        lib32stdc++6 \
        lib32gcc1 \
        lib32z1 \
        lib32ncurses6 \
        libc6-dev \
        openjdk-11-jdk \
        unzip \
        wget \
        zip  > /dev/null && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Install SDK tools
RUN echo "Installing sdk tools ${ANDROID_SDK_TOOLS_VERSION}" && \
    wget --quiet --output-document=sdk-tools.zip \
        https://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_SDK_TOOLS_VERSION}_latest.zip && \
    mkdir --parents "${ANDROID_HOME}/cmdline-tools" && \
    unzip -q sdk-tools.zip -d "${ANDROID_HOME}/cmdline-tools" && \
    mv ${ANDROID_HOME}/cmdline-tools/cmdline-tools ${ANDROID_HOME}/cmdline-tools/latest && \
    rm --force sdk-tools.zip && \
# The `yes` is for accepting all non-standard tool licenses.
    mkdir --parents "${ANDROID_HOME}/.android" && \
    echo "### User Sources for Android SDK Manager" > \
        "${ANDROID_HOME}/.android/repositories.cfg" && \
    yes | sdkmanager --licenses > /dev/null && \
    # Install NDK
    echo "Installing ndk r${ANDROID_NDK_VERSION}" && \
    yes | sdkmanager "ndk;${ANDROID_NDK_VERSION}" --channel=3 && \
    # Install platforms
    echo "Installing platforms" && \
    yes | sdkmanager "platforms;android-${ANDROID_COMPILE_SDK}" > /dev/null && \
    # Install platform tools
    echo "Installing platform tools" && \
    yes | sdkmanager "platform-tools" > /dev/null && \
    # Install build tools
    echo "Installing build tools" && \
    yes | sdkmanager "build-tools;${ANDROID_BUILD_TOOLS}" > /dev/null

# Install Rust toolchains
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable -y
RUN rustup target add \
        armv7-linux-androideabi \
        i686-linux-android \
        aarch64-linux-android \
        x86_64-linux-android

# TODO: Copy sdk license agreement files.