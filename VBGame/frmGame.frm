VERSION 5.00
Begin VB.Form frmGame 
   BackColor       =   &H00000000&
   BorderStyle     =   0  'None
   Caption         =   "Form1"
   ClientHeight    =   9030
   ClientLeft      =   0
   ClientTop       =   0
   ClientWidth     =   9570
   LinkTopic       =   "Form1"
   ScaleHeight     =   9030
   ScaleWidth      =   9570
   ShowInTaskbar   =   0   'False
   StartUpPosition =   3  'Windows Default
   Begin VB.ListBox List1 
      Height          =   1815
      Left            =   6600
      TabIndex        =   1
      Top             =   1440
      Width           =   1815
   End
   Begin VB.TextBox Text1 
      Height          =   1575
      Left            =   1320
      TabIndex        =   0
      Text            =   "Text1"
      Top             =   1320
      Width           =   3615
   End
End
Attribute VB_Name = "frmGame"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = True
Attribute VB_Exposed = False
Dim Exited As Boolean
Dim ddsPic0 As DirectDrawSurface7 'Поверхность под картинку
Dim ddsPic1 As DirectDrawSurface7 'Поверхность под картинку
Dim ddsPic2 As DirectDrawSurface7 'Поверхность под картинку
Dim ddsPic3 As DirectDrawSurface7 'Поверхность под картинку
Dim ddsPic4 As DirectDrawSurface7 'Поверхность под картинку
Dim ddsPic5 As DirectDrawSurface7 'Поверхность под картинку
Dim ddsPic6 As DirectDrawSurface7 'Поверхность под картинку


Private Sub Form_Load()
mdlDirectDraw7.CreateDDFullscreen frmGame.hWnd, 800, 600, 16
Set ddsPic0 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\Obj.bmp") 'Загрузить картинку
'Set ddsPic1 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\spr1.bmp") 'Загрузить картинку
'Set ddsPic2 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\spr2.bmp") 'Загрузить картинку
'Set ddsPic3 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\spr3.bmp") 'Загрузить картинку
'Set ddsPic4 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\spr4.bmp") 'Загрузить картинку
'Set ddsPic5 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\spr5.bmp") 'Загрузить картинку
Exited = False
rc.Bottom = 50
rc.Right = 50
rc.Left = 0
rc.Top = 0
Running = True
While mdlDirectDraw7.Running = True
DoEvents 'Будем жалостливы к системе
Call mdlDirectDraw7.ClearBuffer(ddsBack)  'Чистим полотно

Call ddsBack.BltFast(50, 50, ddsPic0, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)
Call ddsPrimary.Flip(ddsBack, DDFLIP_WAIT)
Wend
'Run
End
End Sub


Public Sub LoadMap1(Address As String)
Dim z As Byte
Dim s As String
ff = FreeFile
Open Address For Input As #ff
i = 0
ReDim mdlStruct.Elm(0)
For yy = 1 To 24
For xx = 1 To 32
z = Val(Input(1, ff))
ReDim Preserve mdlStruct.Elm(i)
mdlStruct.Elm(i).ObjType = z
mdlStruct.Elm(i).kx = xx * 50
mdlStruct.Elm(i).ky = yy * 50
i = i + 1
Next xx
Next yy
Close
End Sub

Public Sub Run11()
Dim ekrt As Integer, ekrl As Integer
ekrt = 0
ekrl = 1
DoEvents
mdlDirectDraw7.ClearBuffer ddsBack 'Чистим полотно
For i = 1 To UBound(mdlStruct.Elm)
'If mdlStruct.Elm(i).ObjType = 0 Then GoTo nxt
rc.Bottom = 50
rc.Right = 50
If mdlStruct.Elm(i).ObjType = 1 Then

ElseIf mdlStruct.Elm(i).ObjType = 2 Then
rc.Left = 250
rc.Top = 0
Call ddsBack.BltFast(mdlStruct.Elm(i).kx - (ekrl * 800), mdlStruct.Elm(i).ky - (ekrt * 600), ddsPic0, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)
ElseIf mdlStruct.Elm(i).ObjType = 3 Then

End If

nxt:
Next i
'Делаем FLIP
Call ddsPrimary.Flip(ddsBack, DDFLIP_WAIT)

End Sub


