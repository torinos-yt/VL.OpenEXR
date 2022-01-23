using System;
using System.Runtime.InteropServices;
using VL.Lib.Basics.Imaging;

namespace OpenEXR
{
    public static class ExrLoader
    {
        enum ExrPixelFormat
        {
            Unknown = -1,
            U32 = 0,
            F16 = 1,
            F32 = 2
        }

        #pragma warning disable CA5393
        [DefaultDllImportSearchPaths(DllImportSearchPath.AssemblyDirectory)]

        [DllImport("../native/VL.OpenEXR.Native.dll", EntryPoint = "load_meta_data")]
        static extern IntPtr LoadExrMeta(string path, out int width, out int height, out ExrPixelFormat format);

        [DllImport("../native/VL.OpenEXR.Native.dll", EntryPoint = "load_exr_f16")]
        static extern IntPtr LoadExrHalf(string path);

        [DllImport("../native/VL.OpenEXR.Native.dll", EntryPoint = "load_exr_f32")]
        static extern IntPtr LoadExrSingle(string path);

        public static byte[] LoadFromPath(string path, out int width, out int height, out PixelFormat format)
        {
            ExrPixelFormat exrFormat;
            LoadExrMeta(path, out width, out height, out exrFormat);

            IntPtr ptr = IntPtr.Zero;

            if(exrFormat == ExrPixelFormat.F16) ptr = LoadExrHalf(path);
            else if(exrFormat == ExrPixelFormat.F32) ptr = LoadExrSingle(path);
            else
            {
                format = PixelFormat.Unknown;
                return new byte[0];
            }
            
            int sizeInBytes = 0;
            (format, sizeInBytes) = exrFormat switch
            {
                ExrPixelFormat.F16 => (PixelFormat.R16G16B16A16F, 2),
                ExrPixelFormat.F32 => (PixelFormat.R32G32B32A32F, 4),
                ExrPixelFormat.U32 => (PixelFormat.Unknown, 4),
                ExrPixelFormat.Unknown => (PixelFormat.Unknown, 0),
                _ => (PixelFormat.Unknown, 0),
            };

            var array = new byte[width * height * 4 * sizeInBytes];
            Marshal.Copy(ptr, array, 0, array.Length);

            Marshal.FreeCoTaskMem(ptr);

            return array;
        }
    }
}
