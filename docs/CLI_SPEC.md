# CLI 命令设计

## 基础命令

```bat
greendev list
greendev search <keyword>
greendev install <tool> [version]
greendev uninstall <tool> [version]
greendev update [tool]
greendev use <tool> <version>
greendev versions <tool>
greendev doctor [tool]
```

## Profile 命令

```bat
greendev profile list
greendev profile show rust-dev
greendev profile install rust-dev
```

也可以简写：

```bat
greendev profile rust-dev
```

## 环境命令

```bat
greendev env generate
greendev env print
greendev env check
greendev env shell
```

## 缓存命令

```bat
greendev cache list
greendev cache clean
greendev cache clean gradle
greendev cache clean cargo
```

## 配置命令

```bat
greendev config get root
greendev config set root D:\Frameworks
greendev config get proxy
greendev config set proxy http://127.0.0.1:7890
```

## 示例

```bat
greendev install java jdk-21
greendev install gradle 8.5
greendev install rust stable
greendev use java jdk-21
greendev profile android-dev
greendev doctor
```
