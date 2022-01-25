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

        [DllImport("../native/VL.OpenEXR.Native.dll")]
        static extern IntPtr load_from_path(string path, out int width, out int height, out ExrPixelFormat format);

        public static byte[] LoadFromPath(string path, out int width, out int height, out PixelFormat format)
        {
            ExrPixelFormat exrFormat;
            IntPtr ptr = load_from_path(path, out width, out height, out exrFormat);

            if(exrFormat == ExrPixelFormat.U32 || exrFormat == ExrPixelFormat.Unknown || ptr == IntPtr.Zero) // TODO : U32 format
            {
                format = PixelFormat.Unknown;
                return new byte[0];
            }
            
            int sizeInBytes = 0;
            (format, sizeInBytes) = exrFormat switch
            {
                ExrPixelFormat.F16 => (PixelFormat.R16G16B16A16F, 2),
                ExrPixelFormat.F32 => (PixelFormat.R32G32B32A32F, 4),
                _ => (PixelFormat.Unknown, 0),
            };

            var array = new byte[width * height * 4 * sizeInBytes];
            Marshal.Copy(ptr, array, 0, array.Length);

            Marshal.FreeCoTaskMem(ptr);

            return array;
        }
    }
}
