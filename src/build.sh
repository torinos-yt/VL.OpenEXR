echo "\033[32mbuild native plugins\033[m"
cd `dirname $0`
cd native
cargo build --target x86_64-pc-windows-gnu --release
cp target/x86_64-pc-windows-gnu/release/vl_openexr_native.dll ../../runtimes/win-x64/native/VL.OpenEXR.Native.dll
echo ""

echo "\033[32mbuild managed plugins\033[m"
cd ../
dotnet.exe build -c release