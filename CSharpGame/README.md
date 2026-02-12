# C# Port of VB6 Game

Target: .NET 7 WinForms (GDI+ graphics placeholder)

Build:
```bash
dotnet build CSharpGame\CSharpGame.csproj
```
Run:
```bash
dotnet run --project CSharpGame\CSharpGame.csproj
```

Next steps:
- Port `mdlStruct.bas` types in detail and move map/sprite data.
- Port `mdlGame.bas` logic (movement, shooting, AI) into `Modules/GameEngine.cs`.
- Port `mdlDirectDraw7.bas` drawing functions to use `System.Drawing` or a higher-performance backend.
