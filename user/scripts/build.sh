#!/bin/bash

# 配置基础地址、步长和链接器脚本路径
BASE_ADDRESS=0x80400000
STEP=0x20000
LINKER="src/linker.ld"

# 获取 src/bin 下的所有应用程序
APPS=$(ls src/bin | sort)
APP_ID=0

# 遍历每个应用程序
for APP in $APPS; do
    # 去掉扩展名，得到应用名称
    APP_NAME=${APP%%.*}

    # 计算当前应用的加载地址
    CURRENT_ADDRESS=$(printf "0x%x" $((BASE_ADDRESS + STEP * APP_ID)))

    # 修改链接器脚本
    ORIGINAL_CONTENT=$(cat $LINKER)
    MODIFIED_CONTENT=$(echo "$ORIGINAL_CONTENT" | sed "s/$(printf "0x%x" $BASE_ADDRESS)/$CURRENT_ADDRESS/g")

    # 写入修改后的链接器脚本
    echo "$MODIFIED_CONTENT" > $LINKER
    # echo "$MODIFIED_CONTENT" > ${APP_NAME}.ld

    # 编译应用程序
    echo "[build.sh] Building application $APP_NAME with start address $CURRENT_ADDRESS"
    cargo build --bin "$APP_NAME" --release

    # 恢复链接器脚本
    echo "$ORIGINAL_CONTENT" > $LINKER

    # 增加应用程序 ID
    APP_ID=$((APP_ID + 1))
done
