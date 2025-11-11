pluginManagement {
    repositories {
        maven("https://mirrors.aliyun.com/repository/public/")
        maven("https://mirrors.aliyun.com/repository/google/")
        maven("https://mirrors.aliyun.com/repository/jcenter/")
        maven("https://mirrors.aliyun.com/repository/gradle-plugins/")
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        maven("https://mirrors.aliyun.com/repository/public/")
        maven("https://mirrors.aliyun.com/repository/google/")
        maven("https://mirrors.aliyun.com/repository/jcenter/")
        google()
        mavenCentral()
    }
}

rootProject.name = "Zhiyun"
include(":app")
include(":core:host")
include(":core:gateway")
include(":core:nearby")
include(":service:agent")
include(":service:builder")
include(":service:editor")
include(":service:explorer")
include(":service:outline")
include(":service:search")
include(":service:terminal")
include(":service:version")
