@echo off
chcp 65001 >nul
title CrossCrypt - 磁盘加密工具
:menu
cls
echo ========================================
echo    CrossCrypt - 磁盘加密工具
echo ========================================
echo.
echo  [1] 创建加密卷
echo  [2] 挂载加密卷
echo  [3] 卸载加密卷
echo  [4] 查看状态
echo  [5] 退出
echo.
echo ========================================
set /p choice=请选择操作 (1-5): 

if "%choice%"=="1" goto create
if "%choice%"=="2" goto mount
if "%choice%"=="3" goto unmount
if "%choice%"=="4" goto status
if "%choice%"=="5" goto exit
if "%choice%"=="q" goto exit
goto menu

:create
cls
echo [创建加密卷]
echo.
set /p device=请输入设备路径 (例如: E:): 
if "%device%"=="" goto create
set /p label=请输入卷标 (可选，直接回车跳过): 
set /p password=请输入密码 (至少8个字符): 
if "%password%"=="" goto create
echo.
echo 警告: 这将格式化设备 %device% 上的所有数据!
set /p confirm=确认继续? (y/N): 
if /i not "%confirm%"=="y" goto menu

crosscrypt.exe create -d "%device%" -l "%label%"
if errorlevel 1 (
    echo.
    echo 创建失败!
) else (
    echo.
    echo 创建成功!
)
pause
goto menu

:mount
cls
echo [挂载加密卷]
echo.
set /p device=请输入设备路径 (例如: E:): 
if "%device%"=="" goto mount
set /p password=请输入密码: 
if "%password%"=="" goto mount
set /p mountpoint=请输入挂载点 (可选，直接回车使用默认): 

crosscrypt.exe mount -d "%device%" -m "%mountpoint%"
if errorlevel 1 (
    echo.
    echo 挂载失败!
) else (
    echo.
    echo 挂载成功!
)
pause
goto menu

:unmount
cls
echo [卸载加密卷]
echo.
set /p target=请输入设备路径或挂载点: 
if "%target%"=="" goto unmount

crosscrypt.exe unmount -t "%target%"
if errorlevel 1 (
    echo.
    echo 卸载失败!
) else (
    echo.
    echo 卸载成功!
)
pause
goto menu

:status
cls
echo [查看状态]
echo.
set /p device=请输入设备路径 (可选，直接回车查看所有): 

if "%device%"=="" (
    crosscrypt.exe status
) else (
    crosscrypt.exe status -d "%device%"
)
pause
goto menu

:exit
echo 感谢使用 CrossCrypt!
timeout /t 2 >nul
exit
