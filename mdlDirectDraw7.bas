Attribute VB_Name = "mdlDirectDraw7"
Option Explicit

'=============================================
' mdlDirectDraw - основы работы с DirectDraw7
' Contact - Antiloop <http://vbdx.chat.ru>
'=============================================

'Объявления объектов DirectDraw
Public dx As New DirectX7 'Объект DirectX
Public dd As DirectDraw7 'Объект DirectDraw
Public ddsPrimary As DirectDrawSurface7 'Главная поверхность
Public ddsBack As DirectDrawSurface7 'Задний буфер
Public ddsd As DDSURFACEDESC2 'Структура с описанием поверхности
Private ddsdStore As DDSURFACEDESC2 'Вспомогательная структура описания
Public rc As RECT 'Структура RECT для блиттинга
Public caps As DDSCAPS2 'Структура с аппаратными возможностями

'Некоторые другие переменные
Public Running As Boolean 'Программа все еще работает?


'Инициализация DirectDraw в полноэкранном режиме
'============================================
Public Sub CreateDDFullscreen(srcHwnd As Long, ByVal dispX As Long, ByVal dispY As Long, ByVal dispColor As Long)
    'Создаем объекты и устанавливаем режим
    Set dd = dx.DirectDrawCreate("")
    Call dd.SetCooperativeLevel(srcHwnd, DDSCL_EXCLUSIVE Or DDSCL_FULLSCREEN Or DDSCL_ALLOWREBOOT)
    Call dd.SetDisplayMode(dispX, dispY, dispColor, 0, DDSDM_DEFAULT)
    
    'Создаем flipping chain
    ddsd.lFlags = DDSD_CAPS Or DDSD_BACKBUFFERCOUNT
    ddsd.ddsCaps.lCaps = DDSCAPS_PRIMARYSURFACE Or DDSCAPS_FLIP Or DDSCAPS_COMPLEX
    ddsd.lBackBufferCount = 1
    Set ddsPrimary = dd.CreateSurface(ddsd)
    
    'Получить BackBuffer
    caps.lCaps = DDSCAPS_BACKBUFFER
    Set ddsBack = ddsPrimary.GetAttachedSurface(caps)
End Sub

'Уничтожить DirectDraw
'====================
Public Sub DestroyDD()
    'Восстанавливаем режим
    Call dd.RestoreDisplayMode
    Call dd.SetCooperativeLevel(0, DDSCL_NORMAL)
    
    'Убиваем поверхности, а ПОТОМ объект DirectDraw
    Set ddsBack = Nothing
    Set ddsPrimary = Nothing
    Set dd = Nothing
End Sub


'Очистка заданного буфера
'========================
    Public Sub ClearBuffer(dds As DirectDrawSurface7)
    Dim Color As Long
    Color = &H4000&    'RGB(0, 0, 0) 'Задаем черный цвет
    dds.GetSurfaceDesc ddsdStore 'Получить описание очищаемой поверхности
    'Заполняем структуру RECT, так, чтобы она охватывала всю поверхность
    With rc
    .Top = 0
    .Left = 0
    .Right = ddsdStore.lWidth 'Вот зачем нам надо было описание поверхности
    .Bottom = ddsdStore.lHeight 'Высота и ширина подгоняются под нее
    End With
    dds.BltColorFill rc, Color 'Заполняем черным цветом область, указанную в RECT
End Sub


'Создание поверхности из картинки в файле
'======================================
Function CreateDDSFromFile(ByVal FileName As String, Optional CKey As Long = 0) As DirectDrawSurface7

    'Объявления
    Dim dds As DirectDrawSurface7 'Временная вспомогательная поверхность
    Dim ddsd As DDSURFACEDESC2 'Описание временной поверхности
    Dim StorePic As Long 'Временное хранилище картинки
    Dim Bmp As Win32.BITMAP 'Тип BITMAP, описывающий растровое изображение
    Dim hDCPicture As Long, hDCSurface As Long 'DC картинки и поверхности
    Dim ddCK As DDCOLORKEY 'Для установки ключевого цвета
    
    Const LR_LOADFROMFILE = &H10
    Const LR_CREATEDIBSECTION = &H2000

    
    'Загружаем картинку и получаем объект картинки
    StorePic = LoadImage(0&, FileName, 0, 0, 0, LR_LOADFROMFILE Or LR_CREATEDIBSECTION) 'Загружаем картинку из файла
    'Получаем описание картинки в структуру BITMAP
    Win32.GetObject StorePic, Len(Bmp), Bmp
    
    'Теперь, создаем поверхность
    ddsd.lFlags = DDSD_CAPS Or DDSD_WIDTH Or DDSD_HEIGHT 'Необходимые флаги
    ddsd.ddsCaps.lCaps = DDSCAPS_OFFSCREENPLAIN 'Поверхность оффскринная
    ddsd.lHeight = Bmp.bmHeight 'Высота поверхности как у картинки
    ddsd.lWidth = Bmp.bmWidth 'Ширина поверхности как у картинки
    'Вызываем метод, создающий поверхность
    Set dds = dd.CreateSurface(ddsd) 'dd - глобальный объект DirectDraw7
    
    'Получаем DC картинки
    hDCPicture = Win32.CreateCompatibleDC(ByVal 0&)
    Win32.SelectObject hDCPicture, StorePic
    'Переводим картинку на поверхность
    Call dds.restore
    hDCSurface = dds.GetDC 'Подготовка к прямому доступу к поверхности
    Call Win32.StretchBlt(hDCSurface, 0, 0, Bmp.bmWidth, Bmp.bmHeight, hDCPicture, 0, 0, Bmp.bmWidth, Bmp.bmHeight, SRCCOPY) 'Это копирует картинку в буфер
    Call dds.ReleaseDC(hDCSurface) 'Конец прямого доступа к поверхности
    Call Win32.DeleteDC(hDCPicture) 'Уничтожить объект картинки - больше не нужен
    
    'Устанавливаем ключевой цвет
    ddCK.low = CKey 'Работает правильно только в 24-битном цвете
    ddCK.high = ddCK.low 'Но для случая 0 пойдет
    Call dds.SetColorKey(DDCKEY_SRCBLT, ddCK)
    
    'Возвращаем объект
    Set CreateDDSFromFile = dds
    'Set dds = Nothing
End Function

