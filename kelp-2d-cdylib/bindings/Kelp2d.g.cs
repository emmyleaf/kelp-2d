// Automatically generated by Interoptopus.

#pragma warning disable 0105
using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Runtime.CompilerServices;
using Kelp2d;
#pragma warning restore 0105

namespace Kelp2d
{
    internal static partial class Native
    {
        public const string NativeLib = "kelp-2d";

        static Native()
        {
        }


        [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_texture_with_data")]
        public static extern FFIError CreateTextureWithData(uint width, uint height, Sliceu8 data, out ulong out_id);

        public static void CreateTextureWithData(uint width, uint height, byte[] data, out ulong out_id)
        {
            unsafe
            {
                fixed (void* ptr_data = data)
                {
                    var data_slice = new Sliceu8(new IntPtr(ptr_data), (ulong) data.Length);
                    var rval = CreateTextureWithData(width, height, data_slice, out out_id);;
                    if (rval != FFIError.Success)
                    {
                        throw new InteropException<FFIError>(rval);
                    }
                }
            }
        }

        [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "initialise")]
        public static extern FFIError Initialise(WindowInfo window, IntPtr imgui_config);

        public static void Initialise_checked(WindowInfo window, IntPtr imgui_config)
        {
            var rval = Initialise(window, imgui_config);;
            if (rval != FFIError.Success)
            {
                throw new InteropException<FFIError>(rval);
            }
        }

        [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "present_frame")]
        public static extern FFIError PresentFrame();

        public static void PresentFrame_checked()
        {
            var rval = PresentFrame();;
            if (rval != FFIError.Success)
            {
                throw new InteropException<FFIError>(rval);
            }
        }

        [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "render_list")]
        public static extern FFIError RenderList(ulong target, Camera camera, ref KelpColor clear, SliceInstanceData instances, SliceInstanceBatch batches);

        public static void RenderList(ulong target, Camera camera, ref KelpColor clear, InstanceData[] instances, InstanceBatch[] batches)
        {
            unsafe
            {
                fixed (void* ptr_instances = instances)
                {
                    var instances_slice = new SliceInstanceData(new IntPtr(ptr_instances), (ulong) instances.Length);
                    fixed (void* ptr_batches = batches)
                    {
                        var batches_slice = new SliceInstanceBatch(new IntPtr(ptr_batches), (ulong) batches.Length);
                        var rval = RenderList(target, camera, ref clear, instances_slice, batches_slice);;
                        if (rval != FFIError.Success)
                        {
                            throw new InteropException<FFIError>(rval);
                        }
                    }
                }
            }
        }

        [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_surface_size")]
        public static extern FFIError SetSurfaceSize(uint width, uint height);

        public static void SetSurfaceSize_checked(uint width, uint height)
        {
            var rval = SetSurfaceSize(width, height);;
            if (rval != FFIError.Success)
            {
                throw new InteropException<FFIError>(rval);
            }
        }

        [DllImport(NativeLib, CallingConvention = CallingConvention.Cdecl, EntryPoint = "uninitialise")]
        public static extern FFIError Uninitialise();

        public static void Uninitialise_checked()
        {
            var rval = Uninitialise();;
            if (rval != FFIError.Success)
            {
                throw new InteropException<FFIError>(rval);
            }
        }

    }

    public enum BlendMode
    {
        ALPHA = 0,
        ADDITIVE = 1,
    }

    public enum WindowType
    {
        Win32 = 0,
        Xlib = 1,
        Wayland = 2,
        AppKit = 3,
    }

    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    internal partial struct Camera
    {
        public float x;
        public float y;
        public float width;
        public float height;
        public float angle;
        public float scale;
    }

    /// A batch of instances to be added to a render pass
    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    internal partial struct InstanceBatch
    {
        public ulong texture;
        public bool smooth;
        public BlendMode blendMode;
        public uint instanceCount;
    }

    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    internal partial struct InstanceData
    {
        public float color0;
        public float color1;
        public float color2;
        public float color3;
        public Transform source;
        public Transform world0;
        public Transform world1;
    }

    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    internal partial struct KelpColor
    {
        public float r;
        public float g;
        public float b;
        public float a;
    }

    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    internal partial struct Transform
    {
        public float renderX;
        public float renderY;
        public float scaleX;
        public float scaleY;
        public float rotation;
        public float originX;
        public float originY;
    }

    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    internal partial struct WindowInfo
    {
        public WindowType windowType;
        public IntPtr windowHandle;
        public IntPtr secondHandle;
        public uint width;
        public uint height;
    }

    /// The main return type for unit returning functions with error handling
    public enum FFIError
    {
        Success = 0,
        Null = 1,
        Panic = 2,
        NoCurrentFrame = 100,
        SwapchainError = 101,
        InvalidTextureId = 102,
        InvalidBindGroupId = 103,
        InvalidPipelineId = 104,
        NoAdapter = 105,
        NoDevice = 106,
        NoImgui = 107,
        ImguiError = 108,
        KelpAlreadyInitialised = 200,
        KelpNotInitialised = 201,
    }

    ///A pointer to an array of data someone else owns which may not be modified.
    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    internal partial struct SliceInstanceBatch
    {
        ///Pointer to start of immutable data.
        IntPtr data;
        ///Number of elements.
        ulong len;
    }

    internal partial struct SliceInstanceBatch : IEnumerable<InstanceBatch>
    {
        public SliceInstanceBatch(GCHandle handle, ulong count)
        {
            this.data = handle.AddrOfPinnedObject();
            this.len = count;
        }
        public SliceInstanceBatch(IntPtr handle, ulong count)
        {
            this.data = handle;
            this.len = count;
        }
        #if (NETSTANDARD2_1_OR_GREATER || NET5_0_OR_GREATER || NETCOREAPP2_1_OR_GREATER)
        public ReadOnlySpan<InstanceBatch> ReadOnlySpan
        {
            get
            {
                unsafe
                {
                    return new ReadOnlySpan<InstanceBatch>(this.data.ToPointer(), (int) this.len);
                }
            }
        }
        #endif
        public InstanceBatch this[int i]
        {
            get
            {
                if (i >= Count) throw new IndexOutOfRangeException();
                unsafe
                {
                    var d = (InstanceBatch*) data.ToPointer();
                    return d[i];
                }
            }
        }
        public InstanceBatch[] Copied
        {
            get
            {
                var rval = new InstanceBatch[len];
                unsafe
                {
                    fixed (void* dst = rval)
                    {
                        #if __INTEROPTOPUS_NEVER
                        #elif NETCOREAPP
                        Unsafe.CopyBlock(dst, data.ToPointer(), (uint) len * (uint) sizeof(InstanceBatch));
                        #else
                        for (var i = 0; i < (int) len; i++) {
                            rval[i] = this[i];
                        }
                        #endif
                    }
                }
                return rval;
            }
        }
        public int Count => (int) len;
        public IEnumerator<InstanceBatch> GetEnumerator()
        {
            for (var i = 0; i < (int)len; ++i)
            {
                yield return this[i];
            }
        }
        IEnumerator IEnumerable.GetEnumerator()
        {
            return this.GetEnumerator();
        }
    }


    ///A pointer to an array of data someone else owns which may not be modified.
    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    internal partial struct SliceInstanceData
    {
        ///Pointer to start of immutable data.
        IntPtr data;
        ///Number of elements.
        ulong len;
    }

    internal partial struct SliceInstanceData : IEnumerable<InstanceData>
    {
        public SliceInstanceData(GCHandle handle, ulong count)
        {
            this.data = handle.AddrOfPinnedObject();
            this.len = count;
        }
        public SliceInstanceData(IntPtr handle, ulong count)
        {
            this.data = handle;
            this.len = count;
        }
        #if (NETSTANDARD2_1_OR_GREATER || NET5_0_OR_GREATER || NETCOREAPP2_1_OR_GREATER)
        public ReadOnlySpan<InstanceData> ReadOnlySpan
        {
            get
            {
                unsafe
                {
                    return new ReadOnlySpan<InstanceData>(this.data.ToPointer(), (int) this.len);
                }
            }
        }
        #endif
        public InstanceData this[int i]
        {
            get
            {
                if (i >= Count) throw new IndexOutOfRangeException();
                var size = Marshal.SizeOf(typeof(InstanceData));
                var ptr = new IntPtr(data.ToInt64() + i * size);
                return Marshal.PtrToStructure<InstanceData>(ptr);
            }
        }
        public InstanceData[] Copied
        {
            get
            {
                var rval = new InstanceData[len];
                for (var i = 0; i < (int) len; i++) {
                    rval[i] = this[i];
                }
                return rval;
            }
        }
        public int Count => (int) len;
        public IEnumerator<InstanceData> GetEnumerator()
        {
            for (var i = 0; i < (int)len; ++i)
            {
                yield return this[i];
            }
        }
        IEnumerator IEnumerable.GetEnumerator()
        {
            return this.GetEnumerator();
        }
    }


    ///A pointer to an array of data someone else owns which may not be modified.
    [Serializable]
    [StructLayout(LayoutKind.Sequential)]
    internal partial struct Sliceu8
    {
        ///Pointer to start of immutable data.
        IntPtr data;
        ///Number of elements.
        ulong len;
    }

    internal partial struct Sliceu8 : IEnumerable<byte>
    {
        public Sliceu8(GCHandle handle, ulong count)
        {
            this.data = handle.AddrOfPinnedObject();
            this.len = count;
        }
        public Sliceu8(IntPtr handle, ulong count)
        {
            this.data = handle;
            this.len = count;
        }
        #if (NETSTANDARD2_1_OR_GREATER || NET5_0_OR_GREATER || NETCOREAPP2_1_OR_GREATER)
        public ReadOnlySpan<byte> ReadOnlySpan
        {
            get
            {
                unsafe
                {
                    return new ReadOnlySpan<byte>(this.data.ToPointer(), (int) this.len);
                }
            }
        }
        #endif
        public byte this[int i]
        {
            get
            {
                if (i >= Count) throw new IndexOutOfRangeException();
                unsafe
                {
                    var d = (byte*) data.ToPointer();
                    return d[i];
                }
            }
        }
        public byte[] Copied
        {
            get
            {
                var rval = new byte[len];
                unsafe
                {
                    fixed (void* dst = rval)
                    {
                        #if __INTEROPTOPUS_NEVER
                        #elif NETCOREAPP
                        Unsafe.CopyBlock(dst, data.ToPointer(), (uint) len * (uint) sizeof(byte));
                        #else
                        for (var i = 0; i < (int) len; i++) {
                            rval[i] = this[i];
                        }
                        #endif
                    }
                }
                return rval;
            }
        }
        public int Count => (int) len;
        public IEnumerator<byte> GetEnumerator()
        {
            for (var i = 0; i < (int)len; ++i)
            {
                yield return this[i];
            }
        }
        IEnumerator IEnumerable.GetEnumerator()
        {
            return this.GetEnumerator();
        }
    }




    public class InteropException<T> : Exception
    {
        public T Error { get; private set; }

        public InteropException(T error): base($"Something went wrong: {error}")
        {
            Error = error;
        }
    }

}
