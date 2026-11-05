Attribute VB_Name = "mdlGame"
Dim np As Byte, kp As Byte
Dim i As Integer
Public Function InRECT(Top As Integer, Left As Integer, Width As Integer, Height As Integer, X As Integer, Y As Integer) As Boolean
InRECT = False
If (Top < X) And (Top + Width > X) Then
If (Left < Y) And (Left + Height > Y) Then
InRECT = True
End If
End If
End Function

Public Sub Move(PlayerIndex As Integer, Up As Integer, Left As Integer)
DoEvents
mdlStruct.Player(PlayerIndex).ZTS = -1
For i = 1 To UBound(mdlStruct.Elm)
If Asc(mdlStruct.Elm(i).ObjType) >= 97 And Asc(mdlStruct.Elm(i).ObjType) <= 121 Then GoTo nxt
If mdlStruct.Elm(i).ObjType = 1 Then
 If InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left, mdlStruct.Player(PlayerIndex).ky + Up) = True Or _
 InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left, mdlStruct.Player(PlayerIndex).ky + Up + 35) = True Or _
 InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left + 35, mdlStruct.Player(PlayerIndex).ky + Up) = True Or _
 InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left + 35, mdlStruct.Player(PlayerIndex).ky + Up + 35) = True Then
 mdlStruct.Player(PlayerIndex).ZTS = i
 End If
GoTo nxt
End If
If mdlStruct.Elm(i).ObjType = 0 Then GoTo nxt
If mdlStruct.Elm(i).ObjType = 3 Then
  If InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left, mdlStruct.Player(PlayerIndex).ky + Up) = True Or _
 InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left, mdlStruct.Player(PlayerIndex).ky + Up + 35) = True Or _
 InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left + 35, mdlStruct.Player(PlayerIndex).ky + Up) = True Or _
 InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left + 35, mdlStruct.Player(PlayerIndex).ky + Up + 35) = True Then
  If mdlStruct.Player(PlayerIndex).Health < 1000 Then
  mdlStruct.Player(PlayerIndex).Health = mdlStruct.Player(PlayerIndex).Health + 1
  End If
 End If
GoTo nxt
End If
If InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left + 15, mdlStruct.Player(PlayerIndex).ky + Up + 15) = True Or _
InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left, mdlStruct.Player(PlayerIndex).ky + Up + 35) = True Or _
InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left + 35, mdlStruct.Player(PlayerIndex).ky + Up + 15) = True Or _
InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left + 35, mdlStruct.Player(PlayerIndex).ky + Up + 35) = True Then
Debug.Print "dgdfg"
Exit Sub
End If
nxt:
Next i
mdlStruct.Player(PlayerIndex).kx = mdlStruct.Player(PlayerIndex).kx + Left
mdlStruct.Player(PlayerIndex).ky = mdlStruct.Player(PlayerIndex).ky + Up

If mdlStruct.Player(0).kx - 800 <= 0 Then
frmStart.ekrl = 0
Else
frmStart.ekrl = 1
End If
If mdlStruct.Player(0).ky - 600 <= 0 Then
frmStart.ekrt = 0
Else
frmStart.ekrt = 1
End If

End Sub

Public Function Move2(PlayerIndex As Integer, Up As Integer, Left As Integer) As Boolean
For i = 1 To UBound(mdlStruct.Elm)
If Asc(mdlStruct.Elm(i).ObjType) >= 97 And Asc(mdlStruct.Elm(i).ObjType) <= 121 Then GoTo nxt
If mdlStruct.Elm(i).ObjType = 1 Then GoTo nxt
If mdlStruct.Elm(i).ObjType = 0 Then GoTo nxt
If mdlStruct.Elm(i).ObjType = 3 Then
GoTo nxt
End If
If InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left + 15, mdlStruct.Player(PlayerIndex).ky + Up + 15) = True Or _
InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left, mdlStruct.Player(PlayerIndex).ky + Up + 35) = True Or _
InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left + 35, mdlStruct.Player(PlayerIndex).ky + Up + 15) = True Or _
InRECT(mdlStruct.Elm(i).kx, mdlStruct.Elm(i).ky, 50, 50, mdlStruct.Player(PlayerIndex).kx + Left + 35, mdlStruct.Player(PlayerIndex).ky + Up + 35) = True Then
Move2 = False
Exit Function
End If
nxt:
Next i
Move2 = True
End Function

Public Function IsPlayer(kx As Integer, ky As Integer, Width As Integer, Height As Integer) As Boolean
For i = 1 To UBound(mdlStruct.Player)
If InRECT(kx, ky, Width, Height, mdlStruct.Player(i).kx, mdlStruct.Player(i).ky) = True Or _
InRECT(kx, ky, Width, Height, mdlStruct.Player(i).kx, mdlStruct.Player(i).ky + 35) = True Or _
InRECT(kx, ky, Width, Height, mdlStruct.Player(i).kx + 35, mdlStruct.Player(i).ky) = True Or _
InRECT(kx, ky, Width, Height, mdlStruct.Player(i).kx + 35, mdlStruct.Player(i).ky + 35) = True Then
IsPlayer = False
Exit Function
End If
Next i
IsPlayer = True
End Function

Public Function IgrokBlizko(PlayerIndex As Integer, EnemyIndex As Integer) As Boolean
For i = frmStart.MaxPl To UBound(mdlStruct.Player)
If InRECT(mdlStruct.Player(PlayerIndex).kx, mdlStruct.Player(PlayerIndex).ky, 35, 35, mdlStruct.Player(i).kx, mdlStruct.Player(i).ky) = True Or _
InRECT(mdlStruct.Player(PlayerIndex).kx, mdlStruct.Player(PlayerIndex).ky, 35, 35, mdlStruct.Player(i).kx, mdlStruct.Player(i).ky + 35) = True Or _
InRECT(mdlStruct.Player(PlayerIndex).kx, mdlStruct.Player(PlayerIndex).ky, 35, 35, mdlStruct.Player(i).kx + 35, mdlStruct.Player(i).ky) = True Or _
InRECT(mdlStruct.Player(PlayerIndex).kx, mdlStruct.Player(PlayerIndex).ky, 35, 35, mdlStruct.Player(i).kx + 35, mdlStruct.Player(i).ky + 35) = True Then
IgrokBlizko = True
Exit Function
End If
Next i
IgrokBlizko = False

End Function
Public Function PoiskBlizIgroka(EnemyIndex As Integer) As Integer
Dim min As Integer, mini As Integer
If mdlStruct.Player(EnemyIndex).kx > -1 And mdlStruct.Player(EnemyIndex).ky > -1 Then
' Определение расстояний до игроков
For i = 1 To frmStart.MaxPl - 1
 If mdlStruct.Player(i).kx > -1 And mdlStruct.Player(i).ky > -1 Then
 mdlStruct.Player(i).rs = Int(Sqr((mdlStruct.Player(j).kx - mdlStruct.Player(i).kx) ^ 2 + (mdlStruct.Player(j).ky - mdlStruct.Player(i).ky) ^ 2))
 End If
Next i
DoEvents
' Определение ближайшего игрока
For i = 1 To frmStart.MaxPl - 1
 If mdlStruct.Player(i).rs < min Then
 min = mdlStruct.Player(i).rs
 mini = i
 End If
Next i
PoiskBlizIgroka = mini
End If
End Function

Public Sub NextStep(EnemyIndex As Integer, PlayerIndex As Integer)
If mdlGame.IgrokBlizko(PlayerIndex, EnemyIndex) = True Then
If mdlStruct.Player(EnemyIndex).nr < 7 Then
mdlStruct.Player(EnemyIndex).nr = 7
ElseIf mdlStruct.Player(EnemyIndex).nr >= 9 Then
mdlStruct.Player(EnemyIndex).nr = 7

Else
mdlStruct.Player(EnemyIndex).nr = mdlStruct.Player(EnemyIndex).nr + 1
End If
mdlStruct.Player(PlayerIndex).Health = mdlStruct.Player(PlayerIndex).Health - 1
 If mdlStruct.Player(PlayerIndex).Health <= 0 Then
 mdlStruct.Player(PlayerIndex).Scores = mdlStruct.Player(PlayerIndex).Scores - 10
 frmStart.SelectPlace PlayerIndex
 mdlStruct.Player(PlayerIndex).Health = 1000
 End If
Exit Sub
End If
If mdlStruct.Player(PlayerIndex).kx < mdlStruct.Player(EnemyIndex).kx And _
mdlStruct.Player(PlayerIndex).ky < mdlStruct.Player(EnemyIndex).ky Then
MakeStep 8, PlayerIndex, EnemyIndex
If mdlStruct.Player(EnemyIndex).nr = 6 Then
mdlStruct.Player(EnemyIndex).nr = 1
Else
mdlStruct.Player(EnemyIndex).nr = mdlStruct.Player(EnemyIndex).nr + 1
End If
ElseIf mdlStruct.Player(PlayerIndex).kx < mdlStruct.Player(EnemyIndex).kx And _
mdlStruct.Player(PlayerIndex).ky = mdlStruct.Player(EnemyIndex).ky Then
MakeStep 7, PlayerIndex, EnemyIndex
If mdlStruct.Player(EnemyIndex).nr = 6 Then
mdlStruct.Player(EnemyIndex).nr = 1
Else
mdlStruct.Player(EnemyIndex).nr = mdlStruct.Player(EnemyIndex).nr + 1
End If
ElseIf mdlStruct.Player(PlayerIndex).kx = mdlStruct.Player(EnemyIndex).kx And _
mdlStruct.Player(PlayerIndex).ky < mdlStruct.Player(EnemyIndex).ky Then
If mdlStruct.Player(EnemyIndex).nr >= 6 Then
mdlStruct.Player(EnemyIndex).nr = 1
Else
mdlStruct.Player(EnemyIndex).nr = mdlStruct.Player(EnemyIndex).nr + 1
End If
MakeStep 1, PlayerIndex, EnemyIndex
ElseIf mdlStruct.Player(PlayerIndex).kx > mdlStruct.Player(EnemyIndex).kx And _
mdlStruct.Player(PlayerIndex).ky > mdlStruct.Player(EnemyIndex).ky Then
If mdlStruct.Player(EnemyIndex).nr >= 6 Then
mdlStruct.Player(EnemyIndex).nr = 1
Else
mdlStruct.Player(EnemyIndex).nr = mdlStruct.Player(EnemyIndex).nr + 1
End If
MakeStep 4, PlayerIndex, EnemyIndex
ElseIf mdlStruct.Player(PlayerIndex).kx = mdlStruct.Player(EnemyIndex).kx And _
mdlStruct.Player(PlayerIndex).ky > mdlStruct.Player(EnemyIndex).ky Then
If mdlStruct.Player(EnemyIndex).nr >= 6 Then
mdlStruct.Player(EnemyIndex).nr = 1
Else
mdlStruct.Player(EnemyIndex).nr = mdlStruct.Player(EnemyIndex).nr + 1
End If
MakeStep 5, PlayerIndex, EnemyIndex

ElseIf mdlStruct.Player(PlayerIndex).kx > mdlStruct.Player(EnemyIndex).kx And _
mdlStruct.Player(PlayerIndex).ky = mdlStruct.Player(EnemyIndex).ky Then
If mdlStruct.Player(EnemyIndex).nr >= 6 Then
mdlStruct.Player(EnemyIndex).nr = 1
Else
mdlStruct.Player(EnemyIndex).nr = mdlStruct.Player(EnemyIndex).nr + 1
End If
MakeStep 3, PlayerIndex, EnemyIndex
ElseIf mdlStruct.Player(PlayerIndex).kx > mdlStruct.Player(EnemyIndex).kx And _
mdlStruct.Player(PlayerIndex).ky < mdlStruct.Player(EnemyIndex).ky Then
If mdlStruct.Player(EnemyIndex).nr >= 6 Then
mdlStruct.Player(EnemyIndex).nr = 1
Else
mdlStruct.Player(EnemyIndex).nr = mdlStruct.Player(EnemyIndex).nr + 1
End If
MakeStep 2, PlayerIndex, EnemyIndex
ElseIf mdlStruct.Player(PlayerIndex).kx < mdlStruct.Player(EnemyIndex).kx And _
mdlStruct.Player(PlayerIndex).ky > mdlStruct.Player(EnemyIndex).ky Then
If mdlStruct.Player(EnemyIndex).nr >= 6 Then
mdlStruct.Player(EnemyIndex).nr = 1
Else
mdlStruct.Player(EnemyIndex).nr = mdlStruct.Player(EnemyIndex).nr + 1
End If
MakeStep 6, PlayerIndex, EnemyIndex

End If
End Sub

Private Sub MakeStep(Pov As Byte, PlayerIndex As Integer, EnemyIndex As Integer)
np = Pov
kp = 0
npv:
If EnemyIndex = 0 Then Exit Sub
'DoEvents
If np = Pov And kp > 0 Then
Else
kp = kp + 1
If Pov = 8 Then
 If Move2(EnemyIndex, -5, -5) = True Then
 mdlStruct.Player(EnemyIndex).nc = 8
 mdlStruct.Player(EnemyIndex).kx = mdlStruct.Player(EnemyIndex).kx - 5
 mdlStruct.Player(EnemyIndex).ky = mdlStruct.Player(EnemyIndex).ky - 5
 ElseIf Move2(EnemyIndex, -5, -5) = False Then
 Pov = 1
 GoTo npv
 End If
ElseIf Pov = 1 Then
 If Move2(EnemyIndex, -10, 0) = True Then
 mdlStruct.Player(EnemyIndex).nc = 1
 mdlStruct.Player(EnemyIndex).ky = mdlStruct.Player(EnemyIndex).ky - 10
 ElseIf Move2(EnemyIndex, -10, 0) = False Then
 Pov = 2
 GoTo npv
 End If
ElseIf Pov = 2 Then
 If Move2(EnemyIndex, -5, 5) = True Then
 mdlStruct.Player(EnemyIndex).nc = 2
 mdlStruct.Player(EnemyIndex).kx = mdlStruct.Player(EnemyIndex).kx + 6
 mdlStruct.Player(EnemyIndex).ky = mdlStruct.Player(EnemyIndex).ky - 6
 ElseIf Move2(EnemyIndex, -5, 5) = False Then
 Pov = 3
 GoTo npv
 End If
ElseIf Pov = 7 Then
 If Move2(EnemyIndex, -10, 0) = True Then
 mdlStruct.Player(EnemyIndex).nc = 7
 mdlStruct.Player(EnemyIndex).kx = mdlStruct.Player(EnemyIndex).kx - 10
 mdlStruct.Player(EnemyIndex).ky = mdlStruct.Player(EnemyIndex).ky - 0
 ElseIf Move2(EnemyIndex, -10, 0) = False Then
 Pov = 8
 GoTo npv
 End If
ElseIf Pov = 4 Then
 If Move2(EnemyIndex, 5, 5) = True Then
 mdlStruct.Player(EnemyIndex).nc = 4
 mdlStruct.Player(EnemyIndex).kx = mdlStruct.Player(EnemyIndex).kx + 6
 mdlStruct.Player(EnemyIndex).ky = mdlStruct.Player(EnemyIndex).ky + 6
 ElseIf Move2(EnemyIndex, 3, 3) = False Then
 Pov = 5
 GoTo npv
 End If
ElseIf Pov = 5 Then
 If Move2(EnemyIndex, 10, 0) = True Then
 mdlStruct.Player(EnemyIndex).nc = 5
 mdlStruct.Player(EnemyIndex).kx = mdlStruct.Player(EnemyIndex).kx + 0
 mdlStruct.Player(EnemyIndex).ky = mdlStruct.Player(EnemyIndex).ky + 10
 ElseIf Move2(EnemyIndex, 10, 0) = False Then
 Pov = 6
 GoTo npv
 End If
ElseIf Pov = 3 Then
 If Move2(EnemyIndex, 0, 10) = True Then
 mdlStruct.Player(EnemyIndex).nc = 3
 mdlStruct.Player(EnemyIndex).kx = mdlStruct.Player(EnemyIndex).kx + 10
 mdlStruct.Player(EnemyIndex).ky = mdlStruct.Player(EnemyIndex).ky + 0
 ElseIf Move2(EnemyIndex, 0, 10) = False Then
 Pov = 4
 GoTo npv
 End If
ElseIf Pov = 2 Then
 If Move2(EnemyIndex, -5, 5) = True Then
 mdlStruct.Player(EnemyIndex).nc = 2
 mdlStruct.Player(EnemyIndex).kx = mdlStruct.Player(EnemyIndex).kx + 5
 mdlStruct.Player(EnemyIndex).ky = mdlStruct.Player(EnemyIndex).ky - 5
 ElseIf Move2(EnemyIndex, -3, 3) = False Then
 Pov = 3
 GoTo npv
 End If
ElseIf Pov = 6 Then
 If Move2(EnemyIndex, 5, -5) = True Then
 mdlStruct.Player(EnemyIndex).nc = 6
 mdlStruct.Player(EnemyIndex).kx = mdlStruct.Player(EnemyIndex).kx - 5
 mdlStruct.Player(EnemyIndex).ky = mdlStruct.Player(EnemyIndex).ky + 5
 ElseIf Move2(EnemyIndex, 3, -3) = False Then
 Pov = 7
 GoTo npv
 End If

End If
End If
End Sub

Public Sub Shoot(PlayerIndex As Integer, Orug As Byte)
Dim ne As Integer
' Orug - Оружие
For i = 1 To UBound(mdlStruct.Pulia)
If mdlStruct.Pulia(i).PType = 0 Then
ProvPov i, PlayerIndex, Orug
Exit Sub
End If
Next i
ne = UBound(mdlStruct.Pulia) + 1
ReDim Preserve mdlStruct.Pulia(ne)
ProvPov ne, PlayerIndex, Orug
End Sub

Private Sub ProvPov(Index As Integer, PlayerIndex As Integer, Orug As Byte)
Dim Pov As Byte
Pov = mdlStruct.Player(PlayerIndex).nc
mdlStruct.Pulia(Index).Napravl = Pov
If Orug = 1 Then
 If Pov = 1 Then
 mdlStruct.Pulia(Index).PType = 1
 mdlStruct.Pulia(Index).kx1 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) + 15
 mdlStruct.Pulia(Index).kx2 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) + 15
 mdlStruct.Pulia(Index).ky1 = mdlStruct.Player(PlayerIndex).ky - (frmStart.ekrt * 600)
 mdlStruct.Pulia(Index).ky2 = mdlStruct.Pulia(Index).ky1 - 20
 ElseIf Pov = 2 Then
 mdlStruct.Pulia(Index).PType = 1
 mdlStruct.Pulia(Index).kx1 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) + 27
 mdlStruct.Pulia(Index).kx2 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) + 15 + 27
 mdlStruct.Pulia(Index).ky1 = mdlStruct.Player(PlayerIndex).ky - (frmStart.ekrt * 600) + 12
 mdlStruct.Pulia(Index).ky2 = mdlStruct.Pulia(Index).ky1 - 15
 ElseIf Pov = 3 Then
 mdlStruct.Pulia(Index).PType = 1
 mdlStruct.Pulia(Index).kx1 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) + 50
 mdlStruct.Pulia(Index).kx2 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) + 20 + 50
 mdlStruct.Pulia(Index).ky1 = mdlStruct.Player(PlayerIndex).ky - (frmStart.ekrt * 600) + 20
 mdlStruct.Pulia(Index).ky2 = mdlStruct.Pulia(Index).ky1 '- 15
 ElseIf Pov = 4 Then
 mdlStruct.Pulia(Index).PType = 1
 mdlStruct.Pulia(Index).kx1 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) + 50
 mdlStruct.Pulia(Index).kx2 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) + 15 + 50
 mdlStruct.Pulia(Index).ky1 = mdlStruct.Player(PlayerIndex).ky - (frmStart.ekrt * 600) + 40
 mdlStruct.Pulia(Index).ky2 = mdlStruct.Pulia(Index).ky1 + 15  '- 15
 ElseIf Pov = 5 Then
 mdlStruct.Pulia(Index).PType = 1
 mdlStruct.Pulia(Index).kx1 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) + 25
 mdlStruct.Pulia(Index).kx2 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) + 25
 mdlStruct.Pulia(Index).ky1 = mdlStruct.Player(PlayerIndex).ky - (frmStart.ekrt * 600) + 40
 mdlStruct.Pulia(Index).ky2 = mdlStruct.Pulia(Index).ky1 + 15  '- 15
 ElseIf Pov = 6 Then
 mdlStruct.Pulia(Index).PType = 1
 mdlStruct.Pulia(Index).kx1 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) - 10
 mdlStruct.Pulia(Index).kx2 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) - 20
 mdlStruct.Pulia(Index).ky1 = mdlStruct.Player(PlayerIndex).ky - (frmStart.ekrt * 600) + 50
 mdlStruct.Pulia(Index).ky2 = mdlStruct.Pulia(Index).ky1 + 15  '- 15
 ElseIf Pov = 7 Then
 mdlStruct.Pulia(Index).PType = 1
 mdlStruct.Pulia(Index).kx1 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) '+ 25
 mdlStruct.Pulia(Index).kx2 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) - 10
 mdlStruct.Pulia(Index).ky1 = mdlStruct.Player(PlayerIndex).ky - (frmStart.ekrt * 600) + 30
 mdlStruct.Pulia(Index).ky2 = mdlStruct.Pulia(Index).ky1 '+ 15  '- 15
 ElseIf Pov = 8 Then
 mdlStruct.Pulia(Index).PType = 1
 mdlStruct.Pulia(Index).kx1 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) - 10 '+ 25
 mdlStruct.Pulia(Index).kx2 = mdlStruct.Player(PlayerIndex).kx - (frmStart.ekrl * 800) - 20
 mdlStruct.Pulia(Index).ky1 = mdlStruct.Player(PlayerIndex).ky - (frmStart.ekrt * 600) - 10
 mdlStruct.Pulia(Index).ky2 = mdlStruct.Pulia(Index).ky1 - 15 '- 15

 End If
End If
End Sub
