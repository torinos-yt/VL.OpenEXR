﻿using System;
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
            bool hasAlpha = true;
            (format, sizeInBytes, hasAlpha) = exrFormat switch
            {
                ExrPixelFormat.F16 => (PixelFormat.R16G16B16A16_Float, 2, true),
                ExrPixelFormat.F32 => (PixelFormat.R32G32B32A32_Float, 4, true),
                ExrPixelFormat.U32 => (PixelFormat.R32G32B32A32_UInt , 4, true),
                ExrPixelFormat.RGBF32 => (PixelFormat.R32G32B32_Float, 4, false),
                _ => (PixelFormat.None, 0, false),
            };

            var array = new byte[width * height * (hasAlpha?4:3) * sizeInBytes];
            Marshal.Copy(ptr, array, 0, array.Length);

            Marshal.FreeCoTaskMem(ptr);

            return array;
        }
    }

    public static class ExrWriter
    {
        #pragma warning disable CA5393
        [DefaultDllImportSearchPaths(DllImportSearchPath.AssemblyDirectory)]

        [DllImport("../native/VL.OpenEXR.Native.dll")]
        static extern void write_texture(string path, int width, int height, ExrPixelFormat format, IntPtr data);
        
        public static void WriteTexture(byte[] data, string path, int width, int height, PixelFormat format)
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
                write_texture(path, width, height, exrFormat, handle.AddrOfPinnedObject());
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
