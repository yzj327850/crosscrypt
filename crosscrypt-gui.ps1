# CrossCrypt GUI - PowerShell Version
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

function Show-MainMenu {
    $form = New-Object System.Windows.Forms.Form
    $form.Text = "CrossCrypt - 磁盘加密工具"
    $form.Size = New-Object System.Drawing.Size(500, 400)
    $form.StartPosition = "CenterScreen"
    $form.FormBorderStyle = "FixedDialog"
    $form.MaximizeBox = $false

    $title = New-Object System.Windows.Forms.Label
    $title.Text = "CrossCrypt"
    $title.Font = New-Object System.Drawing.Font("微软雅黑", 20, [System.Drawing.FontStyle]::Bold)
    $title.AutoSize = $true
    $title.Location = New-Object System.Drawing.Point(150, 20)
    $form.Controls.Add($title)

    $subtitle = New-Object System.Windows.Forms.Label
    $subtitle.Text = "磁盘加密工具"
    $subtitle.Font = New-Object System.Drawing.Font("微软雅黑", 12)
    $subtitle.AutoSize = $true
    $subtitle.Location = New-Object System.Drawing.Point(180, 60)
    $form.Controls.Add($subtitle)

    $btnCreate = New-Object System.Windows.Forms.Button
    $btnCreate.Text = "🔒 创建加密卷"
    $btnCreate.Size = New-Object System.Drawing.Size(200, 40)
    $btnCreate.Location = New-Object System.Drawing.Point(140, 110)
    $btnCreate.Font = New-Object System.Drawing.Font("微软雅黑", 11)
    $btnCreate.Add_Click({ $form.Close(); Show-CreateVolume })
    $form.Controls.Add($btnCreate)

    $btnMount = New-Object System.Windows.Forms.Button
    $btnMount.Text = "🔓 挂载加密卷"
    $btnMount.Size = New-Object System.Drawing.Size(200, 40)
    $btnMount.Location = New-Object System.Drawing.Point(140, 165)
    $btnMount.Font = New-Object System.Drawing.Font("微软雅黑", 11)
    $btnMount.Add_Click({ $form.Close(); Show-MountVolume })
    $form.Controls.Add($btnMount)

    $btnUnmount = New-Object System.Windows.Forms.Button
    $btnUnmount.Text = "⏏️ 卸载加密卷"
    $btnUnmount.Size = New-Object System.Drawing.Size(200, 40)
    $btnUnmount.Location = New-Object System.Drawing.Point(140, 220)
    $btnUnmount.Font = New-Object System.Drawing.Font("微软雅黑", 11)
    $btnUnmount.Add_Click({ $form.Close(); Show-UnmountVolume })
    $form.Controls.Add($btnUnmount)

    $btnStatus = New-Object System.Windows.Forms.Button
    $btnStatus.Text = "📊 查看状态"
    $btnStatus.Size = New-Object System.Drawing.Size(200, 40)
    $btnStatus.Location = New-Object System.Drawing.Point(140, 275)
    $btnStatus.Font = New-Object System.Drawing.Font("微软雅黑", 11)
    $btnStatus.Add_Click({ $form.Close(); Show-Status })
    $form.Controls.Add($btnStatus)

    $form.ShowDialog() | Out-Null
}

function Show-CreateVolume {
    $form = New-Object System.Windows.Forms.Form
    $form.Text = "创建加密卷"
    $form.Size = New-Object System.Drawing.Size(500, 400)
    $form.StartPosition = "CenterScreen"
    $form.FormBorderStyle = "FixedDialog"
    $form.MaximizeBox = $false

    $y = 20
    
    $lblDevice = New-Object System.Windows.Forms.Label
    $lblDevice.Text = "设备路径:"
    $lblDevice.Location = New-Object System.Drawing.Point(20, $y)
    $lblDevice.AutoSize = $true
    $form.Controls.Add($lblDevice)

    $txtDevice = New-Object System.Windows.Forms.TextBox
    $txtDevice.Location = New-Object System.Drawing.Point(120, $y)
    $txtDevice.Size = New-Object System.Drawing.Size(350, 25)
    $txtDevice.Text = "E:"
    $form.Controls.Add($txtDevice)

    $y += 40

    $lblLabel = New-Object System.Windows.Forms.Label
    $lblLabel.Text = "卷标:"
    $lblLabel.Location = New-Object System.Drawing.Point(20, $y)
    $lblLabel.AutoSize = $true
    $form.Controls.Add($lblLabel)

    $txtLabel = New-Object System.Windows.Forms.TextBox
    $txtLabel.Location = New-Object System.Drawing.Point(120, $y)
    $txtLabel.Size = New-Object System.Drawing.Size(350, 25)
    $form.Controls.Add($txtLabel)

    $y += 40

    $lblPassword = New-Object System.Windows.Forms.Label
    $lblPassword.Text = "密码:"
    $lblPassword.Location = New-Object System.Drawing.Point(20, $y)
    $lblPassword.AutoSize = $true
    $form.Controls.Add($lblPassword)

    $txtPassword = New-Object System.Windows.Forms.TextBox
    $txtPassword.Location = New-Object System.Drawing.Point(120, $y)
    $txtPassword.Size = New-Object System.Drawing.Size(350, 25)
    $txtPassword.PasswordChar = '*'
    $form.Controls.Add($txtPassword)

    $y += 40

    $lblConfirm = New-Object System.Windows.Forms.Label
    $lblConfirm.Text = "确认密码:"
    $lblConfirm.Location = New-Object System.Drawing.Point(20, $y)
    $lblConfirm.AutoSize = $true
    $form.Controls.Add($lblConfirm)

    $txtConfirm = New-Object System.Windows.Forms.TextBox
    $txtConfirm.Location = New-Object System.Drawing.Point(120, $y)
    $txtConfirm.Size = New-Object System.Drawing.Size(350, 25)
    $txtConfirm.PasswordChar = '*'
    $form.Controls.Add($txtConfirm)

    $y += 50

    $chkQuick = New-Object System.Windows.Forms.CheckBox
    $chkQuick.Text = "快速格式化 (不加密现有数据)"
    $chkQuick.Location = New-Object System.Drawing.Point(120, $y)
    $chkQuick.AutoSize = $true
    $form.Controls.Add($chkQuick)

    $y += 50

    $lblWarning = New-Object System.Windows.Forms.Label
    $lblWarning.Text = "⚠️ 警告: 这将格式化设备上的所有数据!"
    $lblWarning.ForeColor = [System.Drawing.Color]::Red
    $lblWarning.Location = New-Object System.Drawing.Point(120, $y)
    $lblWarning.AutoSize = $true
    $form.Controls.Add($lblWarning)

    $y += 50

    $btnCreate = New-Object System.Windows.Forms.Button
    $btnCreate.Text = "创建"
    $btnCreate.Size = New-Object System.Drawing.Size(100, 35)
    $btnCreate.Location = New-Object System.Drawing.Point(150, $y)
    $btnCreate.Add_Click({
        if ($txtPassword.Text -ne $txtConfirm.Text) {
            [System.Windows.Forms.MessageBox]::Show("密码不匹配!", "错误", "OK", "Error")
            return
        }
        if ($txtPassword.Text.Length -lt 8) {
            [System.Windows.Forms.MessageBox]::Show("密码至少8个字符!", "错误", "OK", "Error")
            return
        }
        
        $device = $txtDevice.Text
        $label = if ($txtLabel.Text) { "-l `"$($txtLabel.Text)`"" } else { "" }
        $quick = if ($chkQuick.Checked) { "--quick" } else { "" }
        
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = "crosscrypt.exe"
        $psi.Arguments = "create -d `"$device`" $label $quick"
        $psi.UseShellExecute = $false
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError = $true
        
        $proc = [System.Diagnostics.Process]::Start($psi)
        $output = $proc.StandardOutput.ReadToEnd()
        $error = $proc.StandardError.ReadToEnd()
        $proc.WaitForExit()
        
        if ($proc.ExitCode -eq 0) {
            [System.Windows.Forms.MessageBox]::Show("加密卷创建成功!", "成功", "OK", "Information")
        } else {
            [System.Windows.Forms.MessageBox]::Show("创建失败: $error", "错误", "OK", "Error")
        }
    })
    $form.Controls.Add($btnCreate)

    $btnBack = New-Object System.Windows.Forms.Button
    $btnBack.Text = "返回"
    $btnBack.Size = New-Object System.Drawing.Size(100, 35)
    $btnBack.Location = New-Object System.Drawing.Point(270, $y)
    $btnBack.Add_Click({ $form.Close(); Show-MainMenu })
    $form.Controls.Add($btnBack)

    $form.ShowDialog() | Out-Null
}

function Show-MountVolume {
    $form = New-Object System.Windows.Forms.Form
    $form.Text = "挂载加密卷"
    $form.Size = New-Object System.Drawing.Size(500, 300)
    $form.StartPosition = "CenterScreen"
    $form.FormBorderStyle = "FixedDialog"
    $form.MaximizeBox = $false

    $y = 20

    $lblDevice = New-Object System.Windows.Forms.Label
    $lblDevice.Text = "设备路径:"
    $lblDevice.Location = New-Object System.Drawing.Point(20, $y)
    $lblDevice.AutoSize = $true
    $form.Controls.Add($lblDevice)

    $txtDevice = New-Object System.Windows.Forms.TextBox
    $txtDevice.Location = New-Object System.Drawing.Point(120, $y)
    $txtDevice.Size = New-Object System.Drawing.Size(350, 25)
    $txtDevice.Text = "E:"
    $form.Controls.Add($txtDevice)

    $y += 40

    $lblPassword = New-Object System.Windows.Forms.Label
    $lblPassword.Text = "密码:"
    $lblPassword.Location = New-Object System.Drawing.Point(20, $y)
    $lblPassword.AutoSize = $true
    $form.Controls.Add($lblPassword)

    $txtPassword = New-Object System.Windows.Forms.TextBox
    $txtPassword.Location = New-Object System.Drawing.Point(120, $y)
    $txtPassword.Size = New-Object System.Drawing.Size(350, 25)
    $txtPassword.PasswordChar = '*'
    $form.Controls.Add($txtPassword)

    $y += 40

    $lblMount = New-Object System.Windows.Forms.Label
    $lblMount.Text = "挂载点:"
    $lblMount.Location = New-Object System.Drawing.Point(20, $y)
    $lblMount.AutoSize = $true
    $form.Controls.Add($lblMount)

    $txtMount = New-Object System.Windows.Forms.TextBox
    $txtMount.Location = New-Object System.Drawing.Point(120, $y)
    $txtMount.Size = New-Object System.Drawing.Size(350, 25)
    $txtMount.Text = "Z:"
    $form.Controls.Add($txtMount)

    $y += 60

    $btnMount = New-Object System.Windows.Forms.Button
    $btnMount.Text = "挂载"
    $btnMount.Size = New-Object System.Drawing.Size(100, 35)
    $btnMount.Location = New-Object System.Drawing.Point(150, $y)
    $btnMount.Add_Click({
        $device = $txtDevice.Text
        $password = $txtPassword.Text
        $mountpoint = $txtMount.Text
        
        # 这里需要传递密码，但 CLI 不支持直接传递密码参数（安全考虑）
        # 所以这里显示提示
        [System.Windows.Forms.MessageBox]::Show(
            "请使用命令行挂载:\n\ncrosscrypt mount -d $device -m $mountpoint\n\n然后输入密码。",
            "提示", "OK", "Information"
        )
    })
    $form.Controls.Add($btnMount)

    $btnBack = New-Object System.Windows.Forms.Button
    $btnBack.Text = "返回"
    $btnBack.Size = New-Object System.Drawing.Size(100, 35)
    $btnBack.Location = New-Object System.Drawing.Point(270, $y)
    $btnBack.Add_Click({ $form.Close(); Show-MainMenu })
    $form.Controls.Add($btnBack)

    $form.ShowDialog() | Out-Null
}

function Show-UnmountVolume {
    $form = New-Object System.Windows.Forms.Form
    $form.Text = "卸载加密卷"
    $form.Size = New-Object System.Drawing.Size(500, 200)
    $form.StartPosition = "CenterScreen"
    $form.FormBorderStyle = "FixedDialog"
    $form.MaximizeBox = $false

    $y = 20

    $lblTarget = New-Object System.Windows.Forms.Label
    $lblTarget.Text = "设备/挂载点:"
    $lblTarget.Location = New-Object System.Drawing.Point(20, $y)
    $lblTarget.AutoSize = $true
    $form.Controls.Add($lblTarget)

    $txtTarget = New-Object System.Windows.Forms.TextBox
    $txtTarget.Location = New-Object System.Drawing.Point(120, $y)
    $txtTarget.Size = New-Object System.Drawing.Size(350, 25)
    $form.Controls.Add($txtTarget)

    $y += 50

    $btnUnmount = New-Object System.Windows.Forms.Button
    $btnUnmount.Text = "卸载"
    $btnUnmount.Size = New-Object System.Drawing.Size(100, 35)
    $btnUnmount.Location = New-Object System.Drawing.Point(150, $y)
    $btnUnmount.Add_Click({
        $target = $txtTarget.Text
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = "crosscrypt.exe"
        $psi.Arguments = "unmount -t `"$target`""
        $psi.UseShellExecute = $false
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError = $true
        
        $proc = [System.Diagnostics.Process]::Start($psi)
        $output = $proc.StandardOutput.ReadToEnd()
        $error = $proc.StandardError.ReadToEnd()
        $proc.WaitForExit()
        
        if ($proc.ExitCode -eq 0) {
            [System.Windows.Forms.MessageBox]::Show("卸载成功!", "成功", "OK", "Information")
        } else {
            [System.Windows.Forms.MessageBox]::Show("卸载失败: $error", "错误", "OK", "Error")
        }
    })
    $form.Controls.Add($btnUnmount)

    $btnBack = New-Object System.Windows.Forms.Button
    $btnBack.Text = "返回"
    $btnBack.Size = New-Object System.Drawing.Size(100, 35)
    $btnBack.Location = New-Object System.Drawing.Point(270, $y)
    $btnBack.Add_Click({ $form.Close(); Show-MainMenu })
    $form.Controls.Add($btnBack)

    $form.ShowDialog() | Out-Null
}

function Show-Status {
    $form = New-Object System.Windows.Forms.Form
    $form.Text = "查看状态"
    $form.Size = New-Object System.Drawing.Size(500, 300)
    $form.StartPosition = "CenterScreen"
    $form.FormBorderStyle = "FixedDialog"
    $form.MaximizeBox = $false

    $y = 20

    $lblDevice = New-Object System.Windows.Forms.Label
    $lblDevice.Text = "设备路径 (可选):"
    $lblDevice.Location = New-Object System.Drawing.Point(20, $y)
    $lblDevice.AutoSize = $true
    $form.Controls.Add($lblDevice)

    $txtDevice = New-Object System.Windows.Forms.TextBox
    $txtDevice.Location = New-Object System.Drawing.Point(150, $y)
    $txtDevice.Size = New-Object System.Drawing.Size(320, 25)
    $form.Controls.Add($txtDevice)

    $y += 50

    $txtOutput = New-Object System.Windows.Forms.TextBox
    $txtOutput.Multiline = $true
    $txtOutput.ScrollBars = "Vertical"
    $txtOutput.Location = New-Object System.Drawing.Point(20, $y)
    $txtOutput.Size = New-Object System.Drawing.Size(450, 120)
    $txtOutput.ReadOnly = $true
    $form.Controls.Add($txtOutput)

    $y += 140

    $btnCheck = New-Object System.Windows.Forms.Button
    $btnCheck.Text = "查询"
    $btnCheck.Size = New-Object System.Drawing.Size(100, 35)
    $btnCheck.Location = New-Object System.Drawing.Point(100, $y)
    $btnCheck.Add_Click({
        $device = $txtDevice.Text
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = "crosscrypt.exe"
        if ($device) {
            $psi.Arguments = "status -d `"$device`""
        } else {
            $psi.Arguments = "status"
        }
        $psi.UseShellExecute = $false
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError = $true
        
        $proc = [System.Diagnostics.Process]::Start($psi)
        $output = $proc.StandardOutput.ReadToEnd()
        $error = $proc.StandardError.ReadToEnd()
        $proc.WaitForExit()
        
        $txtOutput.Text = $output + $error
    })
    $form.Controls.Add($btnCheck)

    $btnBack = New-Object System.Windows.Forms.Button
    $btnBack.Text = "返回"
    $btnBack.Size = New-Object System.Drawing.Size(100, 35)
    $btnBack.Location = New-Object System.Drawing.Point(270, $y)
    $btnBack.Add_Click({ $form.Close(); Show-MainMenu })
    $form.Controls.Add($btnBack)

    $form.ShowDialog() | Out-Null
}

# 启动主菜单
Show-MainMenu
