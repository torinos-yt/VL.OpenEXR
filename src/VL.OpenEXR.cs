using System;
using System.Runtime.InteropServices;
using Stride.Graphics;

namespace OpenEXR
{
    enum ExrPixelFormat
    {
        Unknown = -1,
        U32 = 0,
        F16 = 1,
        F32 = 2
    }

    public static class ExrLoader
    {
        #pragma warning disable CA5393
        [DefaultDllImportSearchPaths(DllImportSearchPath.AssemblyDirectory)]

        [DllImport("../native/VL.OpenEXR.Native.dll")]
        static extern IntPtr load_from_path(string path, out int width, out int height, out ExrPixelFormat format);

        public static byte[] LoadFromPath(string path, out int width, out int height, out PixelFormat format)
        {
            ExrPixelFormat exrFormat;
            IntPtr ptr = load_from_path(path, out width, out height, out exrFormat);

            if(exrFormat == ExrPixelFormat.Unknown || ptr == IntPtr.Zero)
            {
                format = PixelFormat.None;
                return new byte[0];
            }
            
            int sizeInBytes = 0;
            (format, sizeInBytes) = exrFormat switch
            {
                ExrPixelFormat.F16 => (PixelFormat.R16G16B16A16_Float, 2),
                ExrPixelFormat.F32 => (PixelFormat.R32G32B32A32_Float, 4),
                ExrPixelFormat.U32 => (PixelFormat.R32G32B32A32_UInt, 4),
                _ => (PixelFormat.None, 0),
            };

            var array = new byte[width * height * 4 * sizeInBytes];
            Marshal.Copy(ptr, array, 0, array.Length);

            Marshal.FreeCoTaskMem(ptr);

            return array;
        }
    }
}
