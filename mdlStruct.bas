Attribute VB_Name = "mdlStruct"
Public Type Obj
ObjType As String * 1
kx As Integer
ky As Integer
Dop As Byte
End Type
Public Type Players
PName As String
Health As Integer
kx As Integer
ky As Integer
Orug As Byte
Scores As Long
Kill As Integer
ZTS As Integer
nr As Byte
nc As Byte
Org1 As Byte
Org2 As Byte
Org3 As Byte
Org4 As Byte
Org5 As Byte
rs As Integer
ActiveOrug As Byte
End Type
Public Type TPulia
PType As Byte
kx1 As Integer
ky1 As Integer
kx2 As Integer
ky2 As Integer
Napravl As Byte
End Type
Public Elm() As Obj
Public Player() As Players
Public Pulia() As TPulia
