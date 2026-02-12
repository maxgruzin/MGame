VERSION 5.00
Object = "{248DD890-BB45-11CF-9ABC-0080C7E7B78D}#1.0#0"; "MSWINSCK.OCX"
Begin VB.Form frmStart 
   AutoRedraw      =   -1  'True
   BackColor       =   &H00000000&
   BorderStyle     =   0  'None
   Caption         =   "Game"
   ClientHeight    =   9000
   ClientLeft      =   555
   ClientTop       =   420
   ClientWidth     =   12000
   ControlBox      =   0   'False
   Icon            =   "frmStart.frx":0000
   LinkTopic       =   "Form1"
   MaxButton       =   0   'False
   MinButton       =   0   'False
   MouseIcon       =   "frmStart.frx":0442
   ScaleHeight     =   600
   ScaleMode       =   3  'Pixel
   ScaleWidth      =   800
   WhatsThisHelp   =   -1  'True
   Begin VB.Timer tmrShoot 
      Enabled         =   0   'False
      Interval        =   20
      Left            =   2400
      Top             =   6480
   End
   Begin VB.Timer tmrButtons 
      Enabled         =   0   'False
      Interval        =   35
      Left            =   2400
      Top             =   5880
   End
   Begin VB.Timer tmrEnemysGo 
      Enabled         =   0   'False
      Interval        =   80
      Left            =   4560
      Top             =   6000
   End
   Begin VB.Timer tmrEnemys 
      Enabled         =   0   'False
      Interval        =   2000
      Left            =   4560
      Top             =   5280
   End
   Begin VB.Timer Timer1 
      Enabled         =   0   'False
      Interval        =   50
      Left            =   2400
      Top             =   5280
   End
   Begin VB.PictureBox picPlayer 
      BackColor       =   &H00808080&
      Height          =   735
      Left            =   240
      ScaleHeight     =   675
      ScaleWidth      =   5475
      TabIndex        =   12
      Top             =   1080
      Width           =   5535
      Begin VB.TextBox txtName 
         BackColor       =   &H00C0C0C0&
         BeginProperty Font 
            Name            =   "MS Sans Serif"
            Size            =   13.5
            Charset         =   204
            Weight          =   400
            Underline       =   0   'False
            Italic          =   0   'False
            Strikethrough   =   0   'False
         EndProperty
         ForeColor       =   &H00800000&
         Height          =   405
         Left            =   1200
         MouseIcon       =   "frmStart.frx":074C
         TabIndex        =   14
         Text            =   "Unknown"
         Top             =   120
         Width           =   2775
      End
      Begin VB.Label lblName 
         BackColor       =   &H00808080&
         Caption         =   "Имя"
         BeginProperty Font 
            Name            =   "MS Sans Serif"
            Size            =   12
            Charset         =   204
            Weight          =   700
            Underline       =   0   'False
            Italic          =   0   'False
            Strikethrough   =   0   'False
         EndProperty
         ForeColor       =   &H00FF0000&
         Height          =   375
         Left            =   480
         TabIndex        =   13
         Top             =   120
         Width           =   615
      End
   End
   Begin VB.PictureBox picJoin 
      BackColor       =   &H00808080&
      Height          =   2655
      Left            =   240
      ScaleHeight     =   2595
      ScaleWidth      =   5475
      TabIndex        =   5
      Top             =   1920
      Width           =   5535
      Begin MSWinsockLib.Winsock Client 
         Left            =   1080
         Top             =   1680
         _ExtentX        =   741
         _ExtentY        =   741
         _Version        =   393216
         RemotePort      =   1596
      End
      Begin VB.CommandButton cmdJoin 
         Caption         =   "Подключиться"
         Height          =   495
         Left            =   3000
         TabIndex        =   9
         Top             =   1680
         Width           =   2055
      End
      Begin VB.TextBox txtHost 
         Height          =   375
         Left            =   2160
         TabIndex        =   8
         Top             =   1080
         Width           =   3135
      End
      Begin VB.Label lblIP 
         BackColor       =   &H00808080&
         Caption         =   "IP / HostName"
         BeginProperty Font 
            Name            =   "MS Sans Serif"
            Size            =   12
            Charset         =   204
            Weight          =   700
            Underline       =   0   'False
            Italic          =   0   'False
            Strikethrough   =   0   'False
         EndProperty
         ForeColor       =   &H000000C0&
         Height          =   375
         Left            =   240
         TabIndex        =   7
         Top             =   1080
         Width           =   1815
      End
      Begin VB.Label lblJoin 
         BackColor       =   &H00808080&
         Caption         =   "СОЕДИНЕНИЕ"
         BeginProperty Font 
            Name            =   "MS Sans Serif"
            Size            =   13.5
            Charset         =   204
            Weight          =   400
            Underline       =   0   'False
            Italic          =   0   'False
            Strikethrough   =   0   'False
         EndProperty
         ForeColor       =   &H000000C0&
         Height          =   375
         Left            =   1320
         TabIndex        =   6
         Top             =   120
         Width           =   2295
      End
   End
   Begin VB.CommandButton cmdExit 
      Cancel          =   -1  'True
      Caption         =   "Выход"
      Height          =   495
      Left            =   9960
      TabIndex        =   4
      Top             =   7440
      Width           =   1695
   End
   Begin VB.PictureBox picHost 
      BackColor       =   &H00808080&
      Height          =   6135
      Left            =   6000
      ScaleHeight     =   6075
      ScaleWidth      =   5715
      TabIndex        =   1
      Top             =   1080
      Width           =   5775
      Begin MSWinsockLib.Winsock Server 
         Index           =   0
         Left            =   2040
         Top             =   5160
         _ExtentX        =   741
         _ExtentY        =   741
         _Version        =   393216
         LocalPort       =   1596
      End
      Begin VB.CommandButton cmdCreate 
         Caption         =   "Создать"
         Height          =   495
         Left            =   3600
         TabIndex        =   10
         Top             =   4680
         Width           =   1695
      End
      Begin VB.FileListBox lstMaps 
         Appearance      =   0  'Flat
         BackColor       =   &H00000000&
         ForeColor       =   &H000000FF&
         Height          =   4320
         Left            =   120
         TabIndex        =   2
         Top             =   600
         Width           =   3015
      End
      Begin VB.Label lblInfo 
         BackStyle       =   0  'Transparent
         BorderStyle     =   1  'Fixed Single
         Height          =   3015
         Left            =   3360
         TabIndex        =   11
         Top             =   480
         Width           =   2055
      End
      Begin VB.Label lblHost 
         BackColor       =   &H00808080&
         Caption         =   "ХОСТ ИГРЫ"
         BeginProperty Font 
            Name            =   "MS Sans Serif"
            Size            =   13.5
            Charset         =   204
            Weight          =   400
            Underline       =   0   'False
            Italic          =   0   'False
            Strikethrough   =   0   'False
         EndProperty
         ForeColor       =   &H000000C0&
         Height          =   495
         Left            =   1080
         TabIndex        =   3
         Top             =   120
         Width           =   1815
      End
   End
   Begin VB.PictureBox picLogo 
      Align           =   1  'Align Top
      Appearance      =   0  'Flat
      BackColor       =   &H00000000&
      ForeColor       =   &H80000008&
      Height          =   855
      Left            =   0
      ScaleHeight     =   825
      ScaleWidth      =   11970
      TabIndex        =   0
      Top             =   0
      Width           =   12000
   End
   Begin VB.Label Label1 
      Caption         =   "Label1"
      Height          =   855
      Left            =   4440
      TabIndex        =   15
      Top             =   840
      Visible         =   0   'False
      Width           =   3135
   End
End
Attribute VB_Name = "frmStart"
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
Dim ddsPic7 As DirectDrawSurface7 'Поверхность под тайлы

Dim i As Integer, j As Integer
Dim PI As Integer
Dim WMax As Long
Dim SndOK As Boolean
Public ekrt As Integer, ekrl As Integer
Dim kpt As Long
Public KeyUp As Boolean, KeyDown As Boolean, Attack As Boolean
Public MaxPl As Byte, EnemyCounts As Byte
Const EnemysCount As Integer = 15
Dim TmrPressed As Long

Private Sub Client_Connect()
Client.SendData "Name=" & txtName.Text
End Sub

Private Sub Client_ConnectionRequest(ByVal requestID As Long)
Client.SendData "Name=" & txtName.Text

End Sub

Private Sub Client_DataArrival(ByVal bytesTotal As Long)
Dim s As String
Client.GetData s
If s = "ok" Then
SndOK = True
End If
End Sub

Private Sub Client_Error(ByVal Number As Integer, Description As String, ByVal Scode As Long, ByVal Source As String, ByVal HelpFile As String, ByVal HelpContext As Long, CancelDisplay As Boolean)
MsgBox Description, , "client"
End Sub

Private Sub cmdCreate_Click()
Exited = True
LoadMap lstMaps.Path & "\" & lstMaps.FileName
picHost.Visible = False
picJoin.Visible = False
picLogo.Visible = False
picPlayer.Visible = False
cmdExit.Visible = False
Timer1.Enabled = True
WMax = 0
Server(0).Listen
ReDim Preserve mdlStruct.Player(0)
ReDim Preserve mdlStruct.Pulia(0)
CountPl
If MaxPl > 0 Then
PI = 0
mdlStruct.Player(0).PName = txtName.Text
mdlStruct.Player(0).ZTS = -1
mdlStruct.Player(0).kx = -1
mdlStruct.Player(0).ky = -1
sp 0
tmrEnemys.Enabled = True
Else
MsgBox "Нету возможных мест для игроков", vbCritical, "Map"
End
End If
tmrButtons.Enabled = True
tmrEnemysGo.Enabled = True
tmrShoot.Enabled = True
mdlDirectDraw7.Running = True
'PI = SelectPlace
While mdlDirectDraw7.Running = True
Run
Wend
'Unload frmStart
'frmGame.Show
End Sub
Public Sub sp(Index As Integer)
SelectPlace Index
If mdlStruct.Player(0).kx - 800 <= 0 Then
ekrl = 0
Else
ekrl = 1
End If
If mdlStruct.Player(0).ky - 600 <= 0 Then
ekrt = 0
Else
ekrt = 1
End If
mdlStruct.Player(Index).Health = 1000
mdlStruct.Player(Index).ActiveOrug = 1
End Sub


Public Sub CountPl()
MaxPl = 0
For i = 1 To UBound(mdlStruct.Elm)
If mdlStruct.Elm(i).ObjType = "1" Then
MaxPl = MaxPl + 1
End If
Next i
ReDim mdlStruct.Player(MaxPl - 1 + EnemysCount) '
End Sub
Private Sub cmdExit_Click()
End
End Sub

Private Sub Form_KeyDown(KeyCode As Integer, Shift As Integer)
DoEvents
If KeyCode = vbKeyRight Then
kpt = Timer
 If mdlStruct.Player(PI).nc = 8 Then
 mdlStruct.Player(PI).nc = 1
 Else
 mdlStruct.Player(PI).nc = mdlStruct.Player(PI).nc + 1
 End If
ElseIf KeyCode = vbKeyLeft Then
kpt = Timer
 If mdlStruct.Player(PI).nc = 1 Then
 mdlStruct.Player(PI).nc = 8
 Else
 mdlStruct.Player(PI).nc = mdlStruct.Player(PI).nc - 1
 End If
ElseIf KeyCode = vbKeyUp Then
KeyUp = True
ElseIf KeyCode = vbKeyDown Then
KeyDown = True
ElseIf KeyCode = vbKeyControl Then
 If Timer - TmrPressed > 0.5 Then ' Задержка по времени
 mdlStruct.Player(PI).nr = 9
 mdlGame.Shoot PI, 1
 TmrPressed = Timer
 Else
 mdlStruct.Player(PI).nr = 8

 End If
End If
End Sub

Private Sub Form_KeyUp(KeyCode As Integer, Shift As Integer)
mdlStruct.Player(PI).nr = 1
If KeyCode = vbKeyUp Then
KeyUp = False
ElseIf KeyCode = vbKeyDown Then
KeyDown = False
End If
End Sub

Private Sub Form_Load()

mdlDirectDraw7.CreateDDFullscreen Me.hWnd, 800, 600, 256
Set ddsPic0 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\Obj.bmp", RGB(255, 255, 255)) 'Загрузить картинку
Set ddsPic1 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\spr1.bmp", RGB(255, 255, 255)) 'Загрузить картинку
Set ddsPic2 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\spr2.bmp", RGB(255, 255, 255)) 'Загрузить картинку
Set ddsPic3 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\spr3.bmp", RGB(255, 255, 255)) 'Загрузить картинку
Set ddsPic4 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\spr4.bmp", RGB(255, 255, 255)) 'Загрузить картинку
'Set ddsPic5 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\spr5.bmp", RGB(255, 255, 255)) 'Загрузить картинку
Set ddsPic6 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\Weapons.bmp", RGB(255, 255, 255)) 'Загрузить картинку
Set ddsPic7 = mdlDirectDraw7.CreateDDSFromFile(App.Path & "\Sprites\Tiles.bmp", RGB(255, 255, 255)) 'Загрузить картинку

lstMaps.Path = App.Path & "\Maps"
lstMaps.Pattern = "*.mmp"
Exited = False

End Sub


Private Sub Form_KeyPress(KeyAscii As Integer)
If KeyAscii = vbKeyEscape Then
mdlDirectDraw7.DestroyDD 'Выход
Exited = True
End
End If
End Sub

Private Sub Form_MouseMove(Button As Integer, Shift As Integer, X As Single, Y As Single)
If X > 796 And ekrl = 0 Then
ekrl = 1
ElseIf X < 2 And ekrl = 1 Then
ekrl = 0
End If
If Y > 596 And ekrt = 0 Then
ekrt = 1
ElseIf Y < 2 And ekrt = 1 Then
ekrt = 0
End If
End Sub

Private Sub Form_Unload(Cancel As Integer)
If Exited = False Then mdlDirectDraw7.DestroyDD 'Во избежании ошибок
End Sub

Private Sub picLogo_Resize()
'lblLogo.Width = picLogo.Width
End Sub

Public Sub LoadMap(Address As String)
On Error Resume Next
Dim z As String * 1
Dim s As String
ff = FreeFile
Open Address For Input As #ff
i = 0
ReDim mdlStruct.Elm(0)
For yy = 1 To 24
For xx = 1 To 32
z = Input(1, ff)
ReDim Preserve mdlStruct.Elm(i)
mdlStruct.Elm(i).ObjType = Trim(z)
mdlStruct.Elm(i).kx = (xx - 1) * 50
mdlStruct.Elm(i).ky = (yy - 1) * 50
i = i + 1
Next xx
Line Input #ff, s
Next yy
Close
End Sub

Public Function SelectPlace(PN As Integer) As Integer
' Выбор места размещения игрока
If PN > -1 Then
mdlStruct.Player(PN).ZTS = -1
mdlStruct.Player(PN).kx = -1
mdlStruct.Player(PN).ky = -1
End If
For i = 1 To UBound(mdlStruct.Elm)
If mdlStruct.Elm(i).ObjType = "1" Then
For j = 1 To UBound(mdlStruct.Player)
If mdlStruct.Player(j).ZTS = i Then GoTo nxi
Next j
SelectPlace = i
mdlStruct.Player(PN).kx = mdlStruct.Elm(i).kx
mdlStruct.Player(PN).ky = mdlStruct.Elm(i).ky
mdlStruct.Player(PN).nr = 1
mdlStruct.Player(PN).nc = 3
mdlStruct.Player(PN).ZTS = i
'ekrt = 0
'ekrl = 1
Exit Function
End If
nxi:
Next i

End Function
Public Sub Run()
DoEvents
mdlDirectDraw7.ClearBuffer ddsBack 'Чистим полотно
For i = 1 To UBound(mdlStruct.Elm)
If (mdlStruct.Elm(i).kx - (ekrl * 800)) < 0 Or mdlStruct.Elm(i).ky - (ekrt * 600) < 0 Then GoTo nxt
If (mdlStruct.Elm(i).kx - (ekrl * 800)) > 800 Or mdlStruct.Elm(i).ky - (ekrt * 600) > 600 Then GoTo nxt
If Val(mdlStruct.Elm(i).ObjType) = 9 Then

ElseIf Val(mdlStruct.Elm(i).ObjType) = 2 Then
rc.Bottom = 50
rc.Right = 250
rc.Left = 200
rc.Top = 0
Call ddsBack.BltFast(mdlStruct.Elm(i).kx - (ekrl * 800), mdlStruct.Elm(i).ky - (ekrt * 600), ddsPic0, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)
ElseIf Val(mdlStruct.Elm(i).ObjType) = 3 Then
rc.Bottom = 50
rc.Right = 100
rc.Left = 50
rc.Top = 0
Call ddsBack.BltFast(mdlStruct.Elm(i).kx - (ekrl * 800), mdlStruct.Elm(i).ky - (ekrt * 600), ddsPic0, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)

ElseIf Val(mdlStruct.Elm(i).ObjType) = 4 Then
rc.Bottom = 50
rc.Right = 150
rc.Left = 100
rc.Top = 0
Call ddsBack.BltFast(mdlStruct.Elm(i).kx - (ekrl * 800), mdlStruct.Elm(i).ky - (ekrt * 600), ddsPic0, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)
ElseIf Val(mdlStruct.Elm(i).ObjType) = 5 Then
rc.Bottom = 50
rc.Right = 200
rc.Left = 150
rc.Top = 0
Call ddsBack.BltFast(mdlStruct.Elm(i).kx - (ekrl * 800), mdlStruct.Elm(i).ky - (ekrt * 600), ddsPic0, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)
ElseIf Val(mdlStruct.Elm(i).ObjType) = 6 Then
rc.Bottom = 50
rc.Right = 300
rc.Left = 250
rc.Top = 0
Call ddsBack.BltFast(mdlStruct.Elm(i).kx - (ekrl * 800), mdlStruct.Elm(i).ky - (ekrt * 600), ddsPic0, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)
ElseIf Asc(mdlStruct.Elm(i).ObjType) >= 97 And Asc(mdlStruct.Elm(i).ObjType) <= 121 Then
cs = Asc(mdlStruct.Elm(i).ObjType) - 97
If cs <= 8 Then
rc.Left = cs * 50
rc.Right = rc.Left + 50
rc.Top = 0
rc.Bottom = 50
ElseIf cs > 8 And cs <= 16 Then
rc.Left = (cs - 8) * 50
rc.Right = rc.Left + 50
rc.Top = 50
rc.Bottom = 100
ElseIf cs > 16 And cs <= 24 Then
rc.Left = (cs - 16) * 50
rc.Right = rc.Left + 50
rc.Top = 100
rc.Bottom = 150
End If
Call ddsBack.BltFast(mdlStruct.Elm(i).kx - (ekrl * 800), mdlStruct.Elm(i).ky - (ekrt * 600), ddsPic7, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)
End If

nxt:
Next i
DoEvents
For i = 0 To UBound(mdlStruct.Player)
If mdlStruct.Player(i).PName <> "" Then
If Len(mdlStruct.Player(i).PName) > 1 Then
rc.Right = mdlStruct.Player(i).nc * 50
rc.Left = (mdlStruct.Player(i).nc - 1) * 50
rc.Top = (mdlStruct.Player(i).nr - 1) * 50
rc.Bottom = mdlStruct.Player(i).nr * 50
Call ddsBack.BltFast(mdlStruct.Player(i).kx - (ekrl * 800), mdlStruct.Player(i).ky - (ekrt * 600), ddsPic4, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)
ElseIf mdlStruct.Player(i).PName = "1" Then
rc.Right = mdlStruct.Player(i).nc * 50
rc.Left = (mdlStruct.Player(i).nc - 1) * 50
rc.Top = (mdlStruct.Player(i).nr - 1) * 50
rc.Bottom = mdlStruct.Player(i).nr * 50
Call ddsBack.BltFast(mdlStruct.Player(i).kx - (ekrl * 800), mdlStruct.Player(i).ky - (ekrt * 600), ddsPic1, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)
ElseIf mdlStruct.Player(i).PName = "2" Then
rc.Right = mdlStruct.Player(i).nc * 50
rc.Left = (mdlStruct.Player(i).nc - 1) * 50
rc.Top = (mdlStruct.Player(i).nr - 1) * 50
rc.Bottom = mdlStruct.Player(i).nr * 50
Call ddsBack.BltFast(mdlStruct.Player(i).kx - (ekrl * 800), mdlStruct.Player(i).ky - (ekrt * 600), ddsPic2, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)

End If
End If
Next i
Call ddsBack.SetForeColor(RGB(255, 20, 20)) 'Установить цвет рисования
For i = 1 To UBound(mdlStruct.Pulia)
If mdlStruct.Pulia(i).PType = 1 Then
Call ddsBack.DrawLine(mdlStruct.Pulia(i).kx1, mdlStruct.Pulia(i).ky1, mdlStruct.Pulia(i).kx2, mdlStruct.Pulia(i).ky2)
End If
Next i
Call ddsBack.DrawText(20, 5, "Жизни: " & mdlStruct.Player(PI).Health & "     Очки: " & mdlStruct.Player(PI).Scores & "   Убийств:" & mdlStruct.Player(PI).Kill, False)   'Это текст
If mdlStruct.Player(PI).ActiveOrug = 1 Then
rc.Right = 250
rc.Left = 185
rc.Top = 0
rc.Bottom = 50
Call ddsBack.BltFast(700, 15, ddsPic6, rc, DDBLTFAST_SRCCOLORKEY Or DDBLTFAST_WAIT)
Call ddsBack.DrawText(720, 55, "200", False)  'Это текст

End If
'Делаем FLIP
Call ddsPrimary.Flip(ddsBack, DDFLIP_WAIT)
'Label1.Refresh
frmStart.SetFocus
DoEvents
End Sub

Private Sub Server_ConnectionRequest(Index As Integer, ByVal requestID As Long)
WMax = WMax + 1
Load Server(WMax)
Server(WMax).LocalPort = 0
Server(WMax).Accept requestID
ReDim Preserve mdlStruct.Player(Index)
mdlStruct.Player(Index).ZTS = -1
mdlStruct.Player(Index).kx = -1
mdlStruct.Player(Index).ky = -1
DoEvents

End Sub

Private Sub Server_DataArrival(Index As Integer, ByVal bytesTotal As Long)
Dim Data As String
Server(Index).GetData Data
Select Case Left(Data, 4)
Case "Name"
mdlStruct.Player(Index).PName = Mid(Data, 6)
SelectPlace Index
Server(Index).SendData "ok"
End Select
End Sub

Private Sub Server_Error(Index As Integer, ByVal Number As Integer, Description As String, ByVal Scode As Long, ByVal Source As String, ByVal HelpFile As String, ByVal HelpContext As Long, CancelDisplay As Boolean)
MsgBox Description
End Sub

Private Sub Timer1_Timer()
For i = 1 To UBound(mdlStruct.Elm)
If Val(mdlStruct.Elm(i).ObjType) = 3 Then
 If InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PI).kx + Left, mdlStruct.Player(PI).ky + Up) = True Or _
 InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PI).kx + Left, mdlStruct.Player(PI).ky + Up + 35) = True Or _
 InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PI).kx + Left + 35, mdlStruct.Player(PI).ky + Up) = True Or _
 InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PI).kx + Left + 35, mdlStruct.Player(PI).ky + Up + 35) = True Then
  If mdlStruct.Player(PI).Health < 1000 Then
  mdlStruct.Player(PI).Health = mdlStruct.Player(PI).Health + 1
  End If
 End If
GoTo nxt
End If
nxt:
Next i
End Sub

Private Sub tmrButtons_Timer()
DoEvents
If KeyUp = True Then
 If mdlStruct.Player(PI).nr >= 7 Then
 mdlStruct.Player(PI).nr = 1
 Else
 mdlStruct.Player(PI).nr = mdlStruct.Player(PI).nr + 1
 End If
 
 Select Case mdlStruct.Player(PI).nc
 Case 1
 mdlGame.Move PI, -8, 0
 Case 2
 mdlGame.Move PI, -3, 3
 Case 3
 mdlGame.Move PI, 0, 8
 Case 4
 mdlGame.Move PI, 3, 3
 Case 5
 mdlGame.Move PI, 8, 0
 Case 6
 mdlGame.Move PI, 3, -3
 Case 7
 mdlGame.Move PI, 0, -8
 Case 8
 mdlGame.Move PI, -3, -3
 End Select
End If
If KeyDown = True Then
 If mdlStruct.Player(PI).nr = 1 Then
 mdlStruct.Player(PI).nr = 7
 Else
 mdlStruct.Player(PI).nr = mdlStruct.Player(PI).nr - 1
 End If
 
 Select Case mdlStruct.Player(PI).nc
 Case 1
 mdlGame.Move PI, 4, 0
 Case 2
 mdlGame.Move PI, 2, -2
 Case 3
 mdlGame.Move PI, 0, -4
 Case 4
 mdlGame.Move PI, -2, -2
 Case 5
 mdlGame.Move PI, -2, 0
 Case 6
 mdlGame.Move PI, -2, 2
 Case 7
 mdlGame.Move PI, 0, 4
 Case 8
 mdlGame.Move PI, 2, 2
 End Select
End If

End Sub

Private Sub tmrEnemys_Timer()
Dim ti As Byte
If EnemyCounts < EnemysCount Then
For i = 1 To UBound(mdlStruct.Elm)
If Val(mdlStruct.Elm(i).ObjType) = 2 Then
 If EnemyCounts < EnemysCount Then
  If mdlGame.IsPlayer(mdlStruct.Elm(i + 1).kx, mdlStruct.Elm(i + 1).ky, 50, 50) = True Then
   If mdlStruct.Elm(i + 1).ObjType = 0 Then
   For j = MaxPl To UBound(mdlStruct.Player)
    If mdlStruct.Player(j).PName = "" Then
    ti = Int(Rnd * 2) + 1
    EnemyCounts = EnemyCounts + 1
    mdlStruct.Player(j).PName = ti
    mdlStruct.Player(j).Health = 200
    mdlStruct.Player(j).kx = mdlStruct.Elm(i + 1).kx
    mdlStruct.Player(j).ky = mdlStruct.Elm(i + 1).ky
    mdlStruct.Player(j).nc = 1
    mdlStruct.Player(j).nr = 1
    GoTo nxt:
    End If
   Next j

   ti = Int(Rnd * 2) + 1
   EnemyCounts = EnemyCounts + 1
   mdlStruct.Player(MaxPl + EnemyCounts - 1).PName = ti
   mdlStruct.Player(MaxPl + EnemyCounts - 1).Health = 200
   mdlStruct.Player(MaxPl + EnemyCounts - 1).kx = mdlStruct.Elm(i + 1).kx
   mdlStruct.Player(MaxPl + EnemyCounts - 1).ky = mdlStruct.Elm(i + 1).ky
   mdlStruct.Player(MaxPl + EnemyCounts - 1).nc = 1
   mdlStruct.Player(MaxPl + EnemyCounts - 1).nr = 1
   End If
  End If
 Else
 Exit Sub
 End If
nxt:
If EnemyCounts < EnemysCount Then
 If mdlGame.IsPlayer(mdlStruct.Elm(i - 1).kx, mdlStruct.Elm(i - 1).ky, 50, 50) = True Then
 
  If mdlStruct.Elm(i - 1).ObjType = 0 Then
 ti = Int(Rnd * 2) + 1
 EnemyCounts = EnemyCounts + 1
 'ReDim Preserve mdlStruct.Player(ne)
 mdlStruct.Player(MaxPl + EnemyCounts - 1).PName = ti
 mdlStruct.Player(MaxPl + EnemyCounts - 1).Health = 200
 mdlStruct.Player(MaxPl + EnemyCounts - 1).kx = mdlStruct.Elm(i - 1).kx
 mdlStruct.Player(MaxPl + EnemyCounts - 1).ky = mdlStruct.Elm(i - 1).ky
 mdlStruct.Player(MaxPl + EnemyCounts - 1).nc = 1
 mdlStruct.Player(MaxPl + EnemyCounts - 1).nr = 1
 'EnemyCounts = EnemyCounts + 1
 DoEvents
  End If
 End If
Else
Exit Sub
End If
End If
Next i
End If
End Sub

Private Sub tmrEnemysGo_Timer()
j = 0
For j = MaxPl To UBound(mdlStruct.Player)
mdlGame.NextStep j, mdlGame.PoiskBlizIgroka(j)
'DoEvents
Next j
End Sub

Private Sub tmrShoot_Timer()
Dim e As Integer
For i = 1 To UBound(mdlStruct.Pulia)
If mdlStruct.Pulia(i).PType > 0 Then
If mdlStruct.Pulia(i).Napravl = 1 Then
mdlStruct.Pulia(i).ky1 = mdlStruct.Pulia(i).ky1 - 20
mdlStruct.Pulia(i).ky2 = mdlStruct.Pulia(i).ky2 - 20
ElseIf mdlStruct.Pulia(i).Napravl = 2 Then
mdlStruct.Pulia(i).ky1 = mdlStruct.Pulia(i).ky1 - 15 '+ 12
mdlStruct.Pulia(i).ky2 = mdlStruct.Pulia(i).ky2 - 15 '+ 12
mdlStruct.Pulia(i).kx1 = mdlStruct.Pulia(i).kx1 + 15
mdlStruct.Pulia(i).kx2 = mdlStruct.Pulia(i).kx2 + 15
ElseIf mdlStruct.Pulia(i).Napravl = 3 Then
'mdlStruct.Pulia(i).ky1 = mdlStruct.Pulia(i).ky1 - 15 '+ 12
'mdlStruct.Pulia(i).ky2 = mdlStruct.Pulia(i).ky2 - 15 '+ 12
mdlStruct.Pulia(i).kx1 = mdlStruct.Pulia(i).kx1 + 20
mdlStruct.Pulia(i).kx2 = mdlStruct.Pulia(i).kx2 + 20
ElseIf mdlStruct.Pulia(i).Napravl = 4 Then
mdlStruct.Pulia(i).ky1 = mdlStruct.Pulia(i).ky1 + 15 '+ 12
mdlStruct.Pulia(i).ky2 = mdlStruct.Pulia(i).ky2 + 15 '+ 12
mdlStruct.Pulia(i).kx1 = mdlStruct.Pulia(i).kx1 + 15
mdlStruct.Pulia(i).kx2 = mdlStruct.Pulia(i).kx2 + 15
ElseIf mdlStruct.Pulia(i).Napravl = 5 Then
mdlStruct.Pulia(i).ky1 = mdlStruct.Pulia(i).ky1 + 20
mdlStruct.Pulia(i).ky2 = mdlStruct.Pulia(i).ky2 + 20
ElseIf mdlStruct.Pulia(i).Napravl = 6 Then
mdlStruct.Pulia(i).ky1 = mdlStruct.Pulia(i).ky1 + 15 '+ 12
mdlStruct.Pulia(i).ky2 = mdlStruct.Pulia(i).ky2 + 15 '+ 12
mdlStruct.Pulia(i).kx1 = mdlStruct.Pulia(i).kx1 - 15
mdlStruct.Pulia(i).kx2 = mdlStruct.Pulia(i).kx2 - 15
ElseIf mdlStruct.Pulia(i).Napravl = 7 Then
mdlStruct.Pulia(i).kx1 = mdlStruct.Pulia(i).kx1 - 20
mdlStruct.Pulia(i).kx2 = mdlStruct.Pulia(i).kx2 - 20
ElseIf mdlStruct.Pulia(i).Napravl = 8 Then
mdlStruct.Pulia(i).ky1 = mdlStruct.Pulia(i).ky1 - 15 '+ 12
mdlStruct.Pulia(i).ky2 = mdlStruct.Pulia(i).ky2 - 15 '+ 12
mdlStruct.Pulia(i).kx1 = mdlStruct.Pulia(i).kx1 - 15
mdlStruct.Pulia(i).kx2 = mdlStruct.Pulia(i).kx2 - 15

End If
For e = 0 To UBound(mdlStruct.Elm)
If Asc(mdlStruct.Elm(e).ObjType) >= 97 And Asc(mdlStruct.Elm(e).ObjType) <= 121 Then GoTo nxt
If mdlStruct.Elm(e).ObjType > 0 Then
 If InRECT(mdlStruct.Elm(e).kx, mdlStruct.Elm(e).ky, 50, 50, mdlStruct.Pulia(i).kx1 + (ekrl * 800), mdlStruct.Pulia(i).ky1 + (ekrt * 600)) = True Then
 mdlStruct.Pulia(i).PType = 0
 Exit For
 End If
End If
nxt:
Next e

For e = MaxPl To UBound(mdlStruct.Player)
If InRECT(mdlStruct.Player(e).kx, mdlStruct.Player(e).ky, 50, 50, mdlStruct.Pulia(i).kx1 + (ekrl * 800), mdlStruct.Pulia(i).ky1 + (ekrt * 600)) = True Then
mdlStruct.Pulia(i).PType = 0
mdlStruct.Player(e).Health = mdlStruct.Player(e).Health - 100
If mdlStruct.Player(e).Health <= 0 Then
mdlStruct.Player(e).PName = ""
mdlStruct.Player(PI).Scores = mdlStruct.Player(PI).Scores + 10
EnemyCounts = EnemyCounts - 1
End If
Exit For
End If

Next e
End If
If mdlStruct.Pulia(i).kx2 > 805 Or mdlStruct.Pulia(i).kx1 < 0 Or mdlStruct.Pulia(i).ky2 < 0 Or mdlStruct.Pulia(i).ky1 > 800 Then
 mdlStruct.Pulia(i).PType = 0
 End If
Next i
End Sub
