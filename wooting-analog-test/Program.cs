using System;
using System.Runtime.InteropServices;
using System.Diagnostics;
using System.Windows.Input;
using System.Threading;
using System.Collections.Generic;
using System.Linq;
using Newtonsoft.Json;
using WootingAnalogSDKNET;

namespace analog_sdk_test
{
    class Program
    {
        static void device_event_cb(DeviceEventType eventType, DeviceInfo deviceInfo) {
            Console.WriteLine($"Device event cb called with: {eventType} {deviceInfo}");

            if (eventType == DeviceEventType.Connected) {
                Console.WriteLine($"Attempted read analog in callback, got value: {WootingAnalogSDK.ReadAnalog(30)}");
            }

            Console.WriteLine("Attempting to read connected devices in callback");
            var (devices, infoErr) = WootingAnalogSDK.GetConnectedDevicesInfo();
            foreach (DeviceInfo device in devices)
            {
                Console.WriteLine($"Device info has: {device}, {infoErr}");

            }
            Console.WriteLine("Finished attempting to read connected devices in callback");
        }

        static void TestSpeedN<T>(Stopwatch sw, Func<T> call, string name, int n){
            for (int i = 0; i < n; i++){
                TestSpeed<T>(sw, call, $"Call {i} {name}");
            }
        }
        static void TestSpeed<T>(Stopwatch sw, Func<T> call, string name){
            sw.Reset();
            sw.Start();
            var r = call.Invoke();
            sw.Stop();
            Console.WriteLine($"{name} call time: {sw.ElapsedTicks} ticks, {sw.ElapsedMilliseconds}ms, result: {r}");
        }
    
        
        static List<(KeycodeType, ushort)> code_map = new List<(KeycodeType,ushort)>()
        {
            (KeycodeType.HID, 0x14),
            (KeycodeType.ScanCode1, 0x50),
            //(Native.KeycodeType.VirtualKey, (ushort)VirtualKeys.Numpad7),
            //(Native.KeycodeType.VirtualKeyTranslate, (ushort)VirtualKeys.Down),
        };

        static int _index = 0;
        static void timer_cb(object state)
        {
            _index = (_index + 1) % code_map.Count;
            var ret = WootingAnalogSDK.SetKeycodeMode(code_map[_index].Item1);
            Console.WriteLine($"Switched to {code_map[_index]}, ret: {ret}");
            var (rets, error) = WootingAnalogSDK.ReadAnalog(code_map[_index].Item2);
            Console.WriteLine($"{rets}, {error}");
        }
        
        static void Main(string[] args)
        {
            Console.TreatControlCAsInput = true;
            if (WootingAnalogSDK.IsInitialised){
                Console.WriteLine("SDK Already initialised!");
            }

            var (deviceNo, err) = WootingAnalogSDK.Initialise();
            if (deviceNo >= 0){
                Console.WriteLine($"SDK Successfully initialised with {deviceNo} devices!");
                WootingAnalogSDK.DeviceEvent += device_event_cb;
                //Console.WriteLine($"Yo yo yo 9+10={Native.wooting_analog_add(9,10)}!");
                Stopwatch sw = new Stopwatch();
                TestSpeedN(sw, () => WootingAnalogSDK.ReadAnalog(4), $"read analog HID", 5);
                TestSpeedN(sw, () =>
                {
                    WootingAnalogSDK.SetKeycodeMode(KeycodeType.ScanCode1);
                    return WootingAnalogSDK.ReadAnalog(30);
                }, $"read analog SC", 5);
                TestSpeedN(sw, () => WootingAnalogSDK.ReadFullBuffer(20), $"read_full_buffer", 5);
                var (devices, infoErr) = WootingAnalogSDK.GetConnectedDevicesInfo();
                foreach (DeviceInfo dev in devices)
                {
                    Console.WriteLine($"Device info has: {dev}, {infoErr}");

                }
                //testSpeedN(sw, () => Native.wooting_analog_read_analog_vk(VirtualKeys.A, true), $"Local read analog VK (translate)", 5);
                //testSpeedN(sw, () => Native.wooting_analog_read_analog_vk(VirtualKeys.A, false), $"Local read analog VK (no translate)", 5);
                float val = 0;
                string output = "";
                WootingAnalogSDK.SetKeycodeMode(code_map[_index].Item1);
                Timer t = new Timer(timer_cb, _index, TimeSpan.Zero, TimeSpan.FromSeconds(4) );
                while (true)
                {
                    if (Console.KeyAvailable) {
                        var consoleKey = Console.ReadKey();
                        if (consoleKey.Key == ConsoleKey.C && consoleKey.Modifiers == ConsoleModifiers.Control) {
                            break;
                        }
                    }

                    //var ret = Native.wooting_analog_read_analog_vk(VirtualKeys.A, false);
                    sw.Restart();
                    var (ret, error) = WootingAnalogSDK.ReadAnalog(code_map[_index].Item2);//, devices.First().device_id);
                    sw.Stop();
                    //if (error == Native.AnalogSDKError.Ok && sw.ElapsedMilliseconds > 0)
                    //    Console.WriteLine($"Warning: ReadAnalog {sw.ElapsedMilliseconds}ms");
                    if (!val.Equals(ret)){
                        val = ret;
                        Console.WriteLine($"Val is {val}, e {error}");
                    }

                    sw.Restart();
                    var (read, readErr) = WootingAnalogSDK.ReadFullBuffer(20);
                    sw.Stop();
                    //if (error == Native.AnalogSDKError.Ok && sw.ElapsedMilliseconds > 0)
                    //    Console.WriteLine($"Warning: ReadFullBuffer {sw.ElapsedMilliseconds}ms");
                    string freshOutput = "";
                    if (readErr == WootingAnalogResult.Ok)
                    {
                        foreach (var analog in read)
                        {
                            //VirtualKeys vk = (VirtualKeys)MapVirtualKey((uint)analog.Item1, MAPVK_VSC_TO_VK_EX);
                            //uint scan = MapVirtualKey((uint)VirtualKeys.RightControl, MAPVK_VK_TO_VSC_EX);

                            if (code_map[_index].Item1 == KeycodeType.VirtualKey ||
                                code_map[_index].Item1 == KeycodeType.VirtualKeyTranslate)
                                freshOutput += $"({(VirtualKeys) analog.Item1},{analog.Item2})";
                            else
                                freshOutput += $"(0x{analog.Item1.ToString("X4")},{analog.Item2})";
                                //freshOutput += $"(0x{analog.Item1.ToString("X4")},{vk},{analog.Item2}) - {scan.ToString("X4")}";
                        }
                    }
                    else
                    {
                        freshOutput = $"Read failed with {readErr}";
                    }

                    if (!freshOutput.Equals(output)){
                        Console.WriteLine(output = freshOutput);
                    }
                    //Thread.Sleep(250);
                }

                WootingAnalogSDK.UnInitialise();
            }
            else{
                Console.WriteLine($"SDK couldn't be initialised, err {err}!");
            }
        }
    }
    /// <summary>
    /// Enumeration for virtual keys.
    /// </summary>
    public enum VirtualKeys : byte
    {
        LeftButton = 0x01,
        RightButton = 0x02,
        Cancel = 0x03,
        MiddleButton = 0x04,
        ExtraButton1 = 0x05,
        ExtraButton2 = 0x06,
        Back = 0x08,
        Tab = 0x09,
        Clear = 0x0C,
        Return = 0x0D,
        Shift = 0x10,
        Control = 0x11,
        /// <summary></summary>
        Menu = 0x12,
        /// <summary></summary>
        Pause = 0x13,
        /// <summary></summary>
        CapsLock = 0x14,
        /// <summary></summary>
        Kana = 0x15,
        /// <summary></summary>
        Hangeul = 0x15,
        /// <summary></summary>
        Hangul = 0x15,
        /// <summary></summary>
        Junja = 0x17,
        /// <summary></summary>
        Final = 0x18,
        /// <summary></summary>
        Hanja = 0x19,
        /// <summary></summary>
        Kanji = 0x19,
        /// <summary></summary>
        Escape = 0x1B,
        /// <summary></summary>
        Convert = 0x1C,
        /// <summary></summary>
        NonConvert = 0x1D,
        /// <summary></summary>
        Accept = 0x1E,
        /// <summary></summary>
        ModeChange = 0x1F,
        /// <summary></summary>
        Space = 0x20,
        /// <summary></summary>
        Prior = 0x21,
        /// <summary></summary>
        Next = 0x22,
        /// <summary></summary>
        End = 0x23,
        /// <summary></summary>
        Home = 0x24,
        /// <summary></summary>
        Left = 0x25,
        /// <summary></summary>
        Up = 0x26,
        /// <summary></summary>
        Right = 0x27,
        /// <summary></summary>
        Down = 0x28,
        /// <summary></summary>
        Select = 0x29,
        /// <summary></summary>
        Print = 0x2A,
        /// <summary></summary>
        Execute = 0x2B,
        /// <summary></summary>
        Snapshot = 0x2C,
        /// <summary></summary>
        Insert = 0x2D,
        /// <summary></summary>
        Delete = 0x2E,
        /// <summary></summary>
        Help = 0x2F,
        /// <summary></summary>
        N0 = 0x30,
        /// <summary></summary>
        N1 = 0x31,
        /// <summary></summary>
        N2 = 0x32,
        /// <summary></summary>
        N3 = 0x33,
        /// <summary></summary>
        N4 = 0x34,
        /// <summary></summary>
        N5 = 0x35,
        /// <summary></summary>
        N6 = 0x36,
        /// <summary></summary>
        N7 = 0x37,
        /// <summary></summary>
        N8 = 0x38,
        /// <summary></summary>
        N9 = 0x39,
        /// <summary></summary>
        A = 0x41,
        /// <summary></summary>
        B = 0x42,
        /// <summary></summary>
        C = 0x43,
        /// <summary></summary>
        D = 0x44,
        /// <summary></summary>
        E = 0x45,
        /// <summary></summary>
        F = 0x46,
        /// <summary></summary>
        G = 0x47,
        /// <summary></summary>
        H = 0x48,
        /// <summary></summary>
        I = 0x49,
        /// <summary></summary>
        J = 0x4A,
        /// <summary></summary>
        K = 0x4B,
        /// <summary></summary>
        L = 0x4C,
        /// <summary></summary>
        M = 0x4D,
        /// <summary></summary>
        N = 0x4E,
        /// <summary></summary>
        O = 0x4F,
        /// <summary></summary>
        P = 0x50,
        /// <summary></summary>
        Q = 0x51,
        /// <summary></summary>
        R = 0x52,
        /// <summary></summary>
        S = 0x53,
        /// <summary></summary>
        T = 0x54,
        /// <summary></summary>
        U = 0x55,
        /// <summary></summary>
        V = 0x56,
        /// <summary></summary>
        W = 0x57,
        /// <summary></summary>
        X = 0x58,
        /// <summary></summary>
        Y = 0x59,
        /// <summary></summary>
        Z = 0x5A,
        /// <summary></summary>
        LeftWindows = 0x5B,
        /// <summary></summary>
        RightWindows = 0x5C,
        /// <summary></summary>
        Application = 0x5D,
        /// <summary></summary>
        Sleep = 0x5F,
        /// <summary></summary>
        Numpad0 = 0x60,
        /// <summary></summary>
        Numpad1 = 0x61,
        /// <summary></summary>
        Numpad2 = 0x62,
        /// <summary></summary>
        Numpad3 = 0x63,
        /// <summary></summary>
        Numpad4 = 0x64,
        /// <summary></summary>
        Numpad5 = 0x65,
        /// <summary></summary>
        Numpad6 = 0x66,
        /// <summary></summary>
        Numpad7 = 0x67,
        /// <summary></summary>
        Numpad8 = 0x68,
        /// <summary></summary>
        Numpad9 = 0x69,
        /// <summary></summary>
        Multiply = 0x6A,
        /// <summary></summary>
        Add = 0x6B,
        /// <summary></summary>
        Separator = 0x6C,
        /// <summary></summary>
        Subtract = 0x6D,
        /// <summary></summary>
        Decimal = 0x6E,
        /// <summary></summary>
        Divide = 0x6F,
        /// <summary></summary>
        F1 = 0x70,
        /// <summary></summary>
        F2 = 0x71,
        /// <summary></summary>
        F3 = 0x72,
        /// <summary></summary>
        F4 = 0x73,
        /// <summary></summary>
        F5 = 0x74,
        /// <summary></summary>
        F6 = 0x75,
        /// <summary></summary>
        F7 = 0x76,
        /// <summary></summary>
        F8 = 0x77,
        /// <summary></summary>
        F9 = 0x78,
        /// <summary></summary>
        F10 = 0x79,
        /// <summary></summary>
        F11 = 0x7A,
        /// <summary></summary>
        F12 = 0x7B,
        /// <summary></summary>
        F13 = 0x7C,
        /// <summary></summary>
        F14 = 0x7D,
        /// <summary></summary>
        F15 = 0x7E,
        /// <summary></summary>
        F16 = 0x7F,
        /// <summary></summary>
        F17 = 0x80,
        /// <summary></summary>
        F18 = 0x81,
        /// <summary></summary>
        F19 = 0x82,
        /// <summary></summary>
        F20 = 0x83,
        /// <summary></summary>
        F21 = 0x84,
        /// <summary></summary>
        F22 = 0x85,
        /// <summary></summary>
        F23 = 0x86,
        /// <summary></summary>
        F24 = 0x87,
        /// <summary></summary>
        NumLock = 0x90,
        /// <summary></summary>
        ScrollLock = 0x91,
        /// <summary></summary>
        NEC_Equal = 0x92,
        /// <summary></summary>
        Fujitsu_Jisho = 0x92,
        /// <summary></summary>
        Fujitsu_Masshou = 0x93,
        /// <summary></summary>
        Fujitsu_Touroku = 0x94,
        /// <summary></summary>
        Fujitsu_Loya = 0x95,
        /// <summary></summary>
        Fujitsu_Roya = 0x96,
        /// <summary></summary>
        LeftShift = 0xA0,
        /// <summary></summary>
        RightShift = 0xA1,
        /// <summary></summary>
        LeftControl = 0xA2,
        /// <summary></summary>
        RightControl = 0xA3,
        /// <summary></summary>
        LeftMenu = 0xA4,
        /// <summary></summary>
        RightMenu = 0xA5,
        /// <summary></summary>
        BrowserBack = 0xA6,
        /// <summary></summary>
        BrowserForward = 0xA7,
        /// <summary></summary>
        BrowserRefresh = 0xA8,
        /// <summary></summary>
        BrowserStop = 0xA9,
        /// <summary></summary>
        BrowserSearch = 0xAA,
        /// <summary></summary>
        BrowserFavorites = 0xAB,
        /// <summary></summary>
        BrowserHome = 0xAC,
        /// <summary></summary>
        VolumeMute = 0xAD,
        /// <summary></summary>
        VolumeDown = 0xAE,
        /// <summary></summary>
        VolumeUp = 0xAF,
        /// <summary></summary>
        MediaNextTrack = 0xB0,
        /// <summary></summary>
        MediaPrevTrack = 0xB1,
        /// <summary></summary>
        MediaStop = 0xB2,
        /// <summary></summary>
        MediaPlayPause = 0xB3,
        /// <summary></summary>
        LaunchMail = 0xB4,
        /// <summary></summary>
        LaunchMediaSelect = 0xB5,
        /// <summary></summary>
        LaunchApplication1 = 0xB6,
        /// <summary></summary>
        LaunchApplication2 = 0xB7,
        /// <summary></summary>
        OEM1 = 0xBA,
        /// <summary></summary>
        OEMPlus = 0xBB,
        /// <summary></summary>
        OEMComma = 0xBC,
        /// <summary></summary>
        OEMMinus = 0xBD,
        /// <summary></summary>
        OEMPeriod = 0xBE,
        /// <summary></summary>
        OEM2 = 0xBF,
        /// <summary></summary>
        OEM3 = 0xC0,
        /// <summary></summary>
        OEM4 = 0xDB,
        /// <summary></summary>
        OEM5 = 0xDC,
        /// <summary></summary>
        OEM6 = 0xDD,
        /// <summary></summary>
        OEM7 = 0xDE,
        /// <summary></summary>
        OEM8 = 0xDF,
        /// <summary></summary>
        OEMAX = 0xE1,
        /// <summary></summary>
        OEM102 = 0xE2,
        /// <summary></summary>
        ICOHelp = 0xE3,
        /// <summary></summary>
        ICO00 = 0xE4,
        /// <summary></summary>
        ProcessKey = 0xE5,
        /// <summary></summary>
        ICOClear = 0xE6,
        /// <summary></summary>
        Packet = 0xE7,
        /// <summary></summary>
        OEMReset = 0xE9,
        /// <summary></summary>
        OEMJump = 0xEA,
        /// <summary></summary>
        OEMPA1 = 0xEB,
        /// <summary></summary>
        OEMPA2 = 0xEC,
        /// <summary></summary>
        OEMPA3 = 0xED,
        /// <summary></summary>
        OEMWSCtrl = 0xEE,
        /// <summary></summary>
        OEMCUSel = 0xEF,
        /// <summary></summary>
        OEMATTN = 0xF0,
        /// <summary></summary>
        OEMFinish = 0xF1,
        /// <summary></summary>
        OEMCopy = 0xF2,
        /// <summary></summary>
        OEMAuto = 0xF3,
        /// <summary></summary>
        OEMENLW = 0xF4,
        /// <summary></summary>
        OEMBackTab = 0xF5,
        /// <summary></summary>
        ATTN = 0xF6,
        /// <summary></summary>
        CRSel = 0xF7,
        /// <summary></summary>
        EXSel = 0xF8,
        /// <summary></summary>
        EREOF = 0xF9,
        /// <summary></summary>
        Play = 0xFA,
        /// <summary></summary>
        Zoom = 0xFB,
        /// <summary></summary>
        Noname = 0xFC,
        /// <summary></summary>
        PA1 = 0xFD,
        /// <summary></summary>
        OEMClear = 0xFE
    }
}
