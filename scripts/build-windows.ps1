# Build Windows installer for the desktop application
Write-Host "Building Windows installer..." -ForegroundColor Green

Set-Location xilem-cross-platform\desktop

# Build the application
cargo build --release

# Create distribution directory
$distDir = "..\..\dist\windows"
New-Item -ItemType Directory -Force -Path $distDir | Out-Null

# Copy executable
Copy-Item "target\release\xilem_abcdeez_desktop.exe" "$distDir\GraphLearning.exe"

# Check if WiX Toolset is installed
$wixPath = "${env:ProgramFiles(x86)}\WiX Toolset v3.11\bin"
if (Test-Path $wixPath) {
    Write-Host "Creating MSI installer..." -ForegroundColor Green
    
    # Create WiX configuration
    $wixConfig = @"
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Product Id="*" Name="Graph Learning" Language="1033" Version="1.0.0.0" 
             Manufacturer="Graph Learning Research" UpgradeCode="A2B3C4D5-E6F7-8901-2345-678901234567">
        <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
        <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
        <MediaTemplate />
        
        <Feature Id="ProductFeature" Title="Graph Learning" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
        </Feature>
        
        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="ProgramFilesFolder">
                <Directory Id="INSTALLFOLDER" Name="Graph Learning" />
            </Directory>
            <Directory Id="ProgramMenuFolder">
                <Directory Id="ApplicationProgramsFolder" Name="Graph Learning"/>
            </Directory>
        </Directory>
        
        <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
            <Component Id="ProductComponent">
                <File Source="GraphLearning.exe" />
                <Shortcut Id="ApplicationStartMenuShortcut"
                          Name="Graph Learning"
                          Directory="ApplicationProgramsFolder"
                          WorkingDirectory="INSTALLFOLDER"
                          Icon="GraphLearning.exe"
                          IconIndex="0"
                          Advertise="yes" />
            </Component>
        </ComponentGroup>
        
        <Icon Id="GraphLearning.exe" SourceFile="GraphLearning.exe" />
    </Product>
</Wix>
"@
    
    $wixConfig | Out-File -FilePath "$distDir\GraphLearning.wxs" -Encoding UTF8
    
    # Compile and link WiX
    & "$wixPath\candle.exe" "$distDir\GraphLearning.wxs" -out "$distDir\GraphLearning.wixobj"
    & "$wixPath\light.exe" "$distDir\GraphLearning.wixobj" -out "$distDir\GraphLearning.msi"
    
    Write-Host "MSI installer created: dist\windows\GraphLearning.msi" -ForegroundColor Green
} else {
    Write-Host "WiX Toolset not found. Skipping MSI creation." -ForegroundColor Yellow
    Write-Host "To create MSI installer, install WiX Toolset from: https://wixtoolset.org/" -ForegroundColor Yellow
}

Write-Host "Windows build complete!" -ForegroundColor Green
Write-Host "Executable location: dist\windows\GraphLearning.exe" -ForegroundColor Cyan