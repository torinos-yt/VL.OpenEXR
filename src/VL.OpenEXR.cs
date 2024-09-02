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
        F32 = 2,
        RGBF32 = 3
    }

    public enum ExrEncoding {
        Uncompressed = 0,
        RLE = 1,
        ZIP1 = 2,
        ZIP16 = 3,
        PIZ = 4,
    }

    public static class ExrLoader
    {
        #pragma warning disable CA5393
        [DefaultDllImportSearchPaths(DllImportSearchPath.AssemblyDirectory)]

        [DllImport("../native/VL.OpenEXR.Native.dll")]
        static extern Int32 load_from_path(string path, out int width, out int height, out ExrPixelFormat format, out IntPtr data);

        public static Texture LoadFromPath(string path, GraphicsDevice device, CommandList commandList)
        {
            ExrPixelFormat exrFormat;
            PixelFormat format;
            IntPtr ptr;
            var result = load_from_path(path, out var width, out var height, out exrFormat, out ptr);

            if(result != 0) {
                format = PixelFormat.None;
                return null;
            }

            if(exrFormat == ExrPixelFormat.Unknown || ptr == IntPtr.Zero)
            {
                format = PixelFormat.None;
                return null;
            }

            int sizeInBytes = 0;
            bool hasAlpha = true;
            (format, sizeInBytes, hasAlpha) = exrFormat switch
            {
                ExrPixelFormat.F16 => (PixelFormat.R16G16B16A16_Float, 2, true),
                ExrPixelFormat.F32 => (PixelFormat.R32G32B32A32_Float, 4, true),
                ExrPixelFormat.U32 => (PixelFormat.R32G32B32A32_UInt , 4, true),
                ExrPixelFormat.RGBF32 => (PixelFormat.R32G32B32_Float, 4, false),
                _ => (PixelFormat.None, 0, false),
            };

            var dataPointer = new DataPointer(ptr, width * height * (hasAlpha?4:3) * sizeInBytes);

            var texture = Texture.New2D(device, width, height, format);
            texture.SetData(commandList, dataPointer);

            Marshal.FreeCoTaskMem(ptr);

            return texture;
        }
    }

    public static class ExrWriter
    {
        #pragma warning disable CA5393
        [DefaultDllImportSearchPaths(DllImportSearchPath.AssemblyDirectory)]

        [DllImport("../native/VL.OpenEXR.Native.dll")]
        static extern Int32 write_texture(string path, int width, int height, ExrPixelFormat format, ExrEncoding encoding, IntPtr data);

        public static void WriteTexture(byte[] data, string path, int width, int height, PixelFormat format, ExrEncoding encoding)
        {
            ExrPixelFormat exrFormat = format switch
            {
                PixelFormat.R32G32B32A32_UInt  => ExrPixelFormat.U32,
                PixelFormat.R16G16B16A16_Float => ExrPixelFormat.F16,
                PixelFormat.R32G32B32A32_Float => ExrPixelFormat.F32,
                _ => ExrPixelFormat.Unknown
            };

            if(exrFormat == ExrPixelFormat.Unknown) return;

            GCHandle handle = GCHandle.Alloc(data, GCHandleType.Pinned);
            try
            {
                write_texture(path, width, height, exrFormat, encoding, handle.AddrOfPinnedObject());
            }
            catch(Exception e)
            {
                Console.WriteLine(e);
            }
            finally
            {
                handle.Free();
            }
        }
    }
}
