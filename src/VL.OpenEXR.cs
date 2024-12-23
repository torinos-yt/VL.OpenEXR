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
    }

    public enum ExrEncoding {
        Uncompressed = 0,
        RLE = 1,
        ZIP1 = 2,
        ZIP16 = 3,
        PIZ = 4,
    }

    public enum ExrOutputChannels {
        Rgb = 0,
        Rgba = 1,
    }

    public static class ExrLoader
    {
        #pragma warning disable CA5393
        [DefaultDllImportSearchPaths(DllImportSearchPath.AssemblyDirectory)]

        [DllImport("VL.OpenEXR.Native.dll")]
        static extern Int32 load_from_path(string path, out int width, out int height, out int num_channels, out ExrPixelFormat format, out IntPtr data);

        public static Texture LoadFromPath(string path, GraphicsDevice device)
        {
            ExrPixelFormat exrFormat;
            PixelFormat format;
            IntPtr ptr;
            var result = load_from_path(path, out var width, out var height, out var numChannels, out exrFormat, out ptr);

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
            (format, sizeInBytes) = (exrFormat, numChannels) switch
            {
                (ExrPixelFormat.F16, 4) => (PixelFormat.R16G16B16A16_Float, 2),
                (ExrPixelFormat.F32, 4) => (PixelFormat.R32G32B32A32_Float, 4),
                (ExrPixelFormat.U32, 4) => (PixelFormat.R32G32B32A32_UInt , 4),

                (ExrPixelFormat.F32, 3) => (PixelFormat.R32G32B32_Float, 4),
                (ExrPixelFormat.U32, 3) => (PixelFormat.R32G32B32_UInt , 4),

                (ExrPixelFormat.F16, 2) => (PixelFormat.R16G16_Float, 2),
                (ExrPixelFormat.F32, 2) => (PixelFormat.R32G32_Float, 4),
                (ExrPixelFormat.U32, 2) => (PixelFormat.R32G32_UInt , 4),

                (ExrPixelFormat.F16, 1) => (PixelFormat.R16_Float, 2),
                (ExrPixelFormat.F32, 1) => (PixelFormat.R32_Float, 4),
                (ExrPixelFormat.U32, 1) => (PixelFormat.R32_UInt , 4),
                _ => (PixelFormat.None, 0),
            };

            var rowPitch = width * numChannels * sizeInBytes;

            var texture = Texture.New(
                device,
                TextureDescription.New2D(width, height, format, usage: GraphicsResourceUsage.Immutable),
                new DataBox(ptr, rowPitch, rowPitch * height));

            Marshal.FreeCoTaskMem(ptr);

            return texture;
        }
    }

    public static unsafe class ExrWriter
    {
        #pragma warning disable CA5393
        [DefaultDllImportSearchPaths(DllImportSearchPath.AssemblyDirectory)]

        [DllImport("VL.OpenEXR.Native.dll")]
        static extern int write_texture(string path, int width, int height, ExrPixelFormat format, ExrEncoding encoding, ExrOutputChannels outputChannels, IntPtr data);

        public static int WriteTexture(byte[] data, string path, int width, int height, PixelFormat format, ExrEncoding encoding, ExrOutputChannels outputChannels)
        {
            return WriteTexture((ReadOnlySpan<byte>)data, path, width, height, format, encoding, outputChannels);
        }

        public static int WriteTexture(ReadOnlySpan<byte> data, string path, int width, int height, PixelFormat format, ExrEncoding encoding, ExrOutputChannels outputChannels)
        {
            ExrPixelFormat exrFormat = format switch
            {
                PixelFormat.R32G32B32A32_UInt  => ExrPixelFormat.U32,
                PixelFormat.R16G16B16A16_Float => ExrPixelFormat.F16,
                PixelFormat.R32G32B32A32_Float => ExrPixelFormat.F32,
                _ => ExrPixelFormat.Unknown
            };

            if(exrFormat == ExrPixelFormat.Unknown) return 1; //return with error

            fixed (byte* pointer = data)
            {
                return write_texture(path, width, height, exrFormat, encoding, outputChannels, new IntPtr(pointer));
            }
        }
    }
}
