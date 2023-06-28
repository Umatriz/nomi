import { invoke } from '@tauri-apps/api'
import { useEffect, useState } from 'react'

import { Route, Routes, Link } from 'react-router-dom'

import Main from "./components/app/pages/Main/Main"
import Profile from './components/app/pages/Profile/Profile'

import "./index.css"

function App() {
  return (
    <>
      <ul>
        <li><a href="/">Home</a></li>
        <li><a href="/profile">Profile</a></li>
      </ul>
      <Routes>
        <Route path='/' element={<Main />} />
        <Route path='/profile' element={<Profile />} />
      </Routes>
    </>
  )
}

export default App
